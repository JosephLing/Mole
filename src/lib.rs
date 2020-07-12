pub mod parse;
use liquid;
use log::{error, info, warn};
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
enum ContentType {
    Markdown,
}

fn parse_file(
    content: &str,
    contentType: ContentType,
) -> Result<parse::Article, parse::ParseError> {
    let mut article = parse::Article::parse(content)?;
    info!("{:?} article parsed", article.config.title);
    if contentType == ContentType::Markdown {
        let parser = Parser::new_ext(&article.template, Options::empty());

        // Write to String buffer.
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        article.template = html_output;
        info!("{:?} article converted to html", article.config.title);
        return Ok(article);
    }

    // hack of an error message
    return Err(parse::ParseError::InvalidTemplate);
}

#[test]
fn test_markdown() {
    let a: parse::Article = parse_file(
        "---\nlayout:page\ntitle:cats and dogs---\ncat",
        ContentType::Markdown,
    )
    .unwrap();
    assert_eq!("<p>cat</p>\n", a.template);
}

/// there is probably a nice library for this but ahow
fn search_dir(path: &PathBuf, file_type: &str) -> Vec<PathBuf> {
    let mut f: Vec<PathBuf> = Vec::new();
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            info!(
                "found {:?} in {:?} {:?} {:?}",
                entry.path(),
                path,
                entry.path().extension(),
                file_type
            );
            if let Some(ending) = entry.path().extension() {
                if ending == file_type {
                    f.push(entry.path());
                }
            }
        }
    }
    return f;
}

#[derive(Debug)]
pub struct CustomError(String);

impl From<std::io::Error> for CustomError {
    fn from(e: std::io::Error) -> Self {
        CustomError(e.to_string())
    }
}

impl From<liquid::Error> for CustomError {
    fn from(e: liquid::Error) -> Self {
        CustomError(e.to_string())
    }
}

pub fn read_file(path: &PathBuf) -> Result<String, CustomError> {
    match read_to_string(path)?.parse::<String>() {
        Ok(c) => Ok(c),
        Err(e) => Err(CustomError(e.to_string())),
    }
}

/// note: should only be used for .html files
fn path_file_name_to_string(file_path: &PathBuf) -> Option<String>{
    return Some(file_path
            .file_name()?
            .to_str()?
            .to_owned().replace(".html", ""));
}

type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

/// we are using the eager compiler because:
/// This would be useful in cases where:
/// - Most partial-templates are used
/// - Of the used partial-templates, they are generally used many times.
///
/// this is straight from: https://github.com/cobalt-org/cobalt.rs/blob/7fc4dd8f416e06f396906c0cbd7199b40be0944f/src/cobalt_model/template.rs
/// however I have hacked away some bits of it
/// NOTE: IO path handling won't be as good most likely
fn load_partials_from_path(root: &PathBuf, source: &mut Partials) -> Result<(), CustomError> {
    for file_path in search_dir(&root, "html") {
        info!("found partial {:?} {:?}", root, file_path);
        let content = read_file(&file_path)?;
        //TODO: we want error reporting here not error failure
        // we only want it to fail if we don't find any valid partials (maybe)
        if let Some(rel_path) = path_file_name_to_string(&file_path){
            info!("included added: {:?}", rel_path);
            source.add(rel_path, content);
        } 
    }
    Ok(())
}

fn parse_articles(path: &PathBuf, layout: &Vec<String>, articles: &mut Vec<parse::Article>) {
    for f in search_dir(&path, "md") {
        if let Ok(data) = read_file(&f) {
            match parse_file(&data, ContentType::Markdown) {
                Ok(html) => {
                    if layout.contains(&html.config.layout) {
                        info!("pusehd html");
                        articles.push(html);
                    } else {
                        warn!(
                            "layout {:?} not found in layouts: {:?}",
                            html.config.layout, layout
                        );
                    }
                }
                Err(e) => error!("{:?} in {:?} caused {:?}", f, path, e),
            }
        } else {
            // invalid format in the file
            warn!("invalid file format for {:?}", path);
        }
    }
}

/// a little messy
/// base_layout: default
/// layout: post
/// this will format the base_layout and give it "default" varaible
/// which will be the include for the layout
/// this is to allow jekyll style templating without having to define formatting in layouts
/// it's v. hacky
/// if there is no base_layout just the normal defined layout will be used
fn write_article(path: &PathBuf, art: parse::Article, parser: &liquid::Parser) {
    if let Some(base_layout) = art.config.base_layout {
        info!("creating article {:?} with base layout", art.config.title);
        let template = parser
            .parse(&format!("{{%- include '{0}' -%}}", base_layout))
            .unwrap();
        
        let output = template
            .render(&liquid::object!({
                "content": art.template,
                "default": format!("{{%- include '{0}' -%}}", art.config.layout),
                "config": liquid::object!({
                    "title": art.config.title,
                    "description": art.config.description,
                    "tags": art.config.tags,
                    "categories": art.config.categories,
                    "visible": art.config.visible
                }),
            }))
            .unwrap();
        let mut output_path = path.clone();
        output_path.push(PathBuf::from(art.config.title + ".html"));
        info!("writing to {:?}", output_path);

        let mut file = File::create(output_path).unwrap();
        file.write_all(output.as_bytes()).unwrap();
    } else {
        info!(
            "creating article {:?} without base layout",
            art.config.title
        );
        let template = parser
            .parse(&format!("{{%- include '{0}' -%}}", art.config.layout))
            .unwrap();

        let output = template
            .render(&liquid::object!({
                "content": art.template,
            }))
            .unwrap();

        let mut file = File::create(&path).unwrap();
        file.write_all(output.as_bytes()).unwrap();
    }
}

#[derive(Debug)]
pub struct Build {
    parser: Partials,
    layouts: Vec<String>,
    articles: Vec<parse::Article>,
    source: Vec<parse::Article>,
    tags: HashMap<String, Vec<String>>,
    categories: HashMap<String, Vec<String>>,
}

impl Build {
    pub fn new() -> Self {
        Build {
            // contains all the includes
            parser: Partials::empty(),

            // check that we have the correct layouts
            layouts: Vec::new(),

            // source to output
            articles: Vec::new(),
            source: Vec::new(),

            // we are going to store these so we don't have to
            // recaculate them if we detect changes in the layout or parser/include
            tags: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    pub fn include(mut self, path: &PathBuf) -> Self {
        info!("include {:?}", path);
        if path.exists() && path.is_dir() {
            load_partials_from_path(&path, &mut self.parser).unwrap();
        } else {
            error!("{:?} is not a path or directory", path);
        }
        self
    }

    pub fn layouts(mut self, path: &PathBuf) -> Self {
        info!("include {:?}", path);

        load_partials_from_path(path, &mut self.parser).unwrap();
        for f in search_dir(path, "html") {
            if let Some(rel_path) = path_file_name_to_string(&f){
                self.layouts.push(rel_path);
            }else{
                warn!("could not get layout name");
            }
        }

        self
    }

    pub fn articles(mut self, path: &PathBuf) -> Self {
        info!("articles {:?}", path);

        parse_articles(path, &self.layouts, &mut self.articles);

        self
    }

    /// note: js and css not taken into account :(
    pub fn source(mut self, path: &PathBuf) -> Self {
        info!("source {:?}", path);

        parse_articles(path, &self.layouts, &mut self.source);

        self
    }

    pub fn compile(self, path: &PathBuf) -> Result<(), CustomError> {
        info!("compile {:?}", path);

        if path.exists() {
            let parser = liquid::ParserBuilder::with_stdlib()
                .partials(self.parser)
                .build()
                .unwrap();

            info!("writing to: {:?}", path);
            info!("articles {:?}", self.articles);

            for art in self.articles {
                info!("article: {:?}", art);
                info!("layouts: {:?}", self.layouts);
                write_article(path, art, &parser);
            }
        } else {
            error!("{:?} doesn't exists", path);
        }
        Ok(())
    }
}
