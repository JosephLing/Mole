pub mod article;
use log::{error, info, warn};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
pub mod error;
pub mod parse;
mod util;

pub type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

pub struct Build<'a> {
    includes: Partials,
    articles: Vec<article::Article>,
    layouts: Vec<String>,
    output: &'a PathBuf,
}

impl<'a> Build<'a> {
    pub fn new(output: &'a PathBuf) -> Self {
        Build {
            includes: Partials::empty(),
            layouts: Vec::new(),
            articles: Vec::new(),
            output,
        }
    }

    /// note: includes are hard-coded as .html files
    /// in util:search_dir and util::path_file_name_to_string
    pub fn includes(mut self, dir: &'a PathBuf, layout: bool) -> Self {
        if dir.exists() && dir.is_dir() {
            for file_path in util::search_dir(dir, "html") {
                //TODO: error handling!!!
                let content = util::read_file(&file_path).unwrap();
                if let Some(rel_path) = util::path_file_name_to_string(&file_path) {
                    if layout {
                        info!("new layout {:?}", rel_path);
                    } else {
                        info!("new include {:?}", rel_path);
                    }
                    self.includes.add(&rel_path, content);
                    if layout {
                        self.layouts.push(rel_path);
                    }
                }
            }
        // info!("found {:?} templates", v.len());
        } else {
            error!("{:?} is not a path or directory", &dir);
        }
        self
    }

    pub fn articles(mut self, temp: &'a Vec<&'a PathBuf>) -> Self {
        for dir in temp {
            info!("looking for markdown articles in {:?}", dir);
            if dir.exists() && dir.is_dir() {
                if self.layouts.is_empty() {
                    panic!(
                "empty layout list, please load in layout template files before parsing articles"
            );
                } else {
                    for f in util::search_dir(&dir, "md") {
                        let p = &util::path_file_name_to_string(&f).unwrap();
                        if !p.starts_with("_") {
                            if let Ok(cat) = File::open(&f) {
                                let buffered = BufReader::new(cat);
                                // &std::io::BufReader<std::path::PathBuf>
                                match article::Article::parse(buffered, p) {
                                    Ok(art) => self.articles.push(art),
                                    Err(e) => error!("{:?}", e),
                                }
                            } else {
                                error!("could not read {:?}", &f);
                            }
                        }
                    }
                }
            } else {
                error!("{:?} is not a path or directory", dir);
            }
        }

        self
    }

    /// TODO: ignore _files
    pub fn sass(self, dir: &'a PathBuf, load_paths: &Vec<&Path>) -> Self {
        if dir.exists() && dir.is_dir() {
            for f in util::search_dir(dir, "scss") {
                info!("{:?}", f);
                if let Ok(data) = read_to_string(&f) {
                    match grass::from_string(
                        data,
                        &grass::Options::default().load_paths(load_paths),
                    ) {
                        Ok(css) => {
                            let mut output_path = self.output.clone();
                            output_path
                                .push(Path::new(&util::path_file_name_to_string(&f).unwrap()));
                            info!("writing css to {:?}", output_path);

                            let mut file = File::create(output_path).unwrap();
                            file.write_all(css.as_bytes()).unwrap();
                            //TODO: write to disk
                        }
                        Err(e) => warn!("parsing sccs {:?} caused {:?}", &f, e),
                    }
                } else {
                    warn!("soemthing went wrong");
                }
            }
        } else {
            warn!("{:?} is not a path or directory, .css files will be copied across but no .sccs compiling will happen", dir);
        }
        self
    }

    pub fn run(self) {
        let mut global_articles: Vec<&liquid::Object> = Vec::new();
        let mut global_tags: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut global_cats: HashMap<&str, Vec<&str>> = HashMap::new();

        let parser = liquid::ParserBuilder::with_stdlib()
            .partials(self.includes)
            .build()
            .unwrap();
        for obj in &self.articles {
            // global_tags.push(&obj.config_liquid.tags);
            // global_cats.push(&obj.config_liquid.cats);
            global_articles.push(&obj.config_liquid);
            for tag in &obj.config.tags {
                global_tags.entry(tag).or_insert(Vec::new()).push(&obj.url);
            }

            for cat in &obj.config.categories {
                global_cats.entry(cat).or_insert(Vec::new()).push(&obj.url);
            }
        }

        let global = liquid::object!({
            "articles": global_articles,
            "tags": global_tags,
            "cats": global_cats,
        });

        info!("{:?}", self.layouts);

        for obj in self.articles {
            //TODO: make this be the url
            let mut output_path = self.output.clone();
            output_path.push(PathBuf::from(&obj.url));
            info!("writing to {:?}", output_path);

            match obj.true_render(&global, &parser) {
                Ok(output) => {
                    let mut file = File::create(output_path).unwrap();
                    file.write_all(output.as_bytes()).unwrap();
                }
                Err(e) => error!("{}", e),
            }
        }
    }
}
