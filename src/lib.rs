pub mod parse;
use log::{error, info, warn};
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::path::{PathBuf, Path};
mod error;
mod util;

pub type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

pub struct Build {
    includes: Partials,
    articles: Vec<parse::Article>,
    layouts: Vec<String>,
    output: PathBuf,
    parser: Option<liquid::Parser>,
}

/**
build::new("_output/")
    .includes(vec![self.include, self.layouts])
    .articles(vec![self.source, self.articles])
    .sass(self.sass, vec![self.load_paths])
    .compile(self.dest)
*/

impl Build {
    pub fn new(path: PathBuf) -> Self {
        Build {
            includes: Partials::empty(),
            layouts: Vec::new(),
            articles: Vec::new(),
            output: path,
            parser: None
        }
    }

    /// note: includes are hard-coded as .html files
    /// in util:search_dir and util::path_file_name_to_string
    pub fn includes(mut self, temp: &Vec<PathBuf>, layout: bool) -> Self {
        for dir in temp {
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
                error!("{:?} is not a path or directory", dir);
            }
        }
        self
    }

    pub fn articles(mut self, temp: &Vec<PathBuf>) -> Self {
        self.parser = Some(liquid::ParserBuilder::with_stdlib()
        .partials(self.includes)
        .build()
        .unwrap());
        for dir in temp {
            info!("looking for markdown articles in {:?}", dir);
            if dir.exists() && dir.is_dir() {
                if self.layouts.is_empty() {
                    panic!(
                "empty layout list, please load in layout template files before parsing articles"
            );
                } else {
                    for f in util::search_dir(&dir, "md") {
                        if let Ok(data) = util::read_file(&f) {
                            self.articles.push(parse::Article::parse(&data).unwrap().pre_render(&self.parser.unwrap()));
                        } else {
                            // invalid format in the file
                            warn!("invalid file format for {:?}", dir);
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
    pub fn sass(self, temp: &Vec<PathBuf>) -> Self {
        for dir in temp {
            if dir.exists() && dir.is_dir() {
                for f in util::search_dir(dir, "scss") {
                    info!("{:?}", f);
                    if let Ok(data) = read_to_string(&f) {
                        match grass::from_string(data) {
                            Ok(css) => {
                                let mut output_path = self.output.clone();
                                output_path.push(Path::new(&util::path_file_name_to_string(&f).unwrap()));
                                info!("writing css to {:?}", output_path);

                                let mut file = File::create(output_path).unwrap();
                                file.write_all(css.as_bytes()).unwrap();
                                //TODO: write to disk
                            }
                            Err(e) => warn!("parsing sccs {:?} caused {:?}", f, e),
                        }
                    } else {
                        warn!("soemthing went wrong");
                    }
                }
            } else {
                warn!("{:?} is not a path or directory, .css files will be copied across but no .sccs compiling will happen", dir);
            }
        }
        self
    }

    pub fn run(mut self) {
        let mut foo: Vec<&liquid::Object> = Vec::new();

        for obj in &self.articles {
            foo.push(&obj.config_liquid);
        }

        let global = liquid::object!({
            "articles": foo,
        });

        
        info!("{:?}", self.layouts);

        for obj in self.articles {
            // write the result to p
            let output: String = obj.render(&global, &self.parser.unwrap()).unwrap();

            //TODO: make this be the url
            let mut output_path = self.output.clone();
            output_path.push(PathBuf::from(&obj.url));
            info!("writing to {:?}", output_path);

            let mut file = File::create(output_path).unwrap();
            file.write_all(output.as_bytes()).unwrap();
        }
    }
}
