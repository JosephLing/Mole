use crate::error::CustomError;
use log::warn;
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Empty,
    NoHeader,
    InvalidHeader(String), // we will want more info here
    InvalidTemplate,       // more info on this
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub layout: String,
    pub base_layout: String,
    pub title: String,
    pub description: Option<String>,
    pub permalink: String,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub visible: bool,
}

impl Config {
    /// For example:
    ///
    /// layout: post
    /// title:  Test2
    /// description: asdf
    ///
    /// TODO these:
    /// categories: programming
    /// tags: [github, github-pages, jekyll]

    // Key factor is that they can read in any order as long as they exists
    fn parse(data: &str) -> Result<Config, ParseError> {
        // going with simple to code and hopefully fast instead of fancy and dynamic

        let mut pieces: HashMap<String, String> = HashMap::new();
        for line in data.split("\n") {
            let line = line.trim();
            if !line.is_empty() {
                let temp: Vec<&str> = line.split(":").collect();
                if let Some(key) = temp.get(0) {
                    if let Some(value) = temp.get(1) {
                        pieces.insert(key.to_string(), value.trim().to_string());
                    } else {
                        warn!("no value was found for {:?} in line {:?}", key, line);
                    }
                } else {
                    warn!("no key value pair found on {:?}", line);
                }
            }
        }
        if let Some(layout) = pieces.get("layout") {
            if let Some(title) = pieces.get("title") {
                // duplicate code ugh...
                // one potential solution would be to use serde_yaml but that's yaml....
                // these could be matches
                Ok(Config {
                    // TODO: how do we handle spaces at the start of layout and title e.g. layout: page won't match "page" as it would be " page"
                    layout: layout.to_string(),
                    title: title.to_string(),
                    description: if let Some(description) = pieces.get("description") {
                        Some(description.to_string())
                    } else {
                        None
                    },
                    // TODO: read in base_layout as an option
                    base_layout: match pieces.get("base_layout") {
                        Some(b) => b.to_string(),
                        None => "default".to_string(),
                    },
                    permalink: if let Some(permalink) = pieces.get("permalink") {
                        permalink.to_string()
                    } else {
                        String::from("")
                    },
                    // TODO: these as lists would be really nice.... potentailly thinking about spinning out the json example again maybe....
                    categories: None,
                    tags: None,
                    visible: match pieces.get("visible") {
                        Some(b) => match b.parse::<bool>() {
                            Ok(b) => b,
                            Err(_) => true,
                        },
                        None => true,
                    },
                })
            } else {
                Err(ParseError::InvalidHeader(String::from(
                    "no title found in config",
                )))
            }
        } else {
            Err(ParseError::InvalidHeader(String::from(
                "no layout found in config",
            )))
        }
    }
}

#[test]
fn test_config_layout() {
    assert_eq!(
        Some(ParseError::InvalidHeader(String::from(
            "no layout found in config"
        ))),
        Config::parse("").err()
    );
}

#[test]
fn test_config_title() {
    assert_eq!(
        Some(ParseError::InvalidHeader(String::from(
            "no title found in config"
        ))),
        Config::parse("layout: page").err()
    );
}

#[test]
fn test_config() {
    assert_eq!(
        Config {
            layout: "page".to_string(),
            base_layout: "default".to_string(),
            title: "hello world".to_string(),
            description: None,
            permalink: "".to_string(),
            categories: None,
            tags: None,
            visible: true
        },
        Config::parse("layout:page\ntitle:hello world").unwrap()
    );
}

#[test]
fn test_config_carrige_returns() {
    assert_eq!(
        Config {
            layout: "page".to_string(),
            base_layout: "default".to_string(),
            title: "hello world".to_string(),
            description: None,
            permalink: "".to_string(),
            categories: None,
            tags: None,
            visible: true
        },
        Config::parse("layout:page\r\ntitle:hello world").unwrap()
    );
}

#[derive(Debug)]
pub struct Article {
    pub template: String,
    pub config: Config,
    pub url: String,
    pub config_liquid: liquid::Object,
}

fn to_string_vector(v: &Vec<&str>, start: usize) -> Option<String> {
    if v.is_empty() {
        return None;
    }
    let mut output = String::from("");
    for e in start..v.len() {
        if e != start {
            let temp = "---".to_owned() + v.get(e).unwrap();
            output += &temp;
        } else {
            output += v.get(e).unwrap();
        }
    }

    return Some(output);
}

impl Article {
    /// header is in a --- --- block with new lines
    /// the rest of the doc is template in markdown
    pub fn parse(md: &str) -> Result<Article, ParseError> {
        if md.is_empty() {
            return Err(ParseError::Empty);
        }

        let lines: Vec<&str> = md.split("---").collect();
        if let Some(config) = lines.get(1) {
            if let Some(content) = to_string_vector(&lines, 2) {
                if content.starts_with("-") {
                    return Err(ParseError::InvalidHeader(
                        "only 3 dashes allowed for config header".to_string(),
                    ));
                }
                let config = Config::parse(config)?;
                
                // markdown parsing NOTE: we are assuming that we are dealing with markdown hear!!!
                let template = content.trim().to_string();
                let parser = Parser::new_ext(&template, Options::empty());

                // Write to String buffer.
                let mut template = String::new();
                html::push_html(&mut template, parser);

                let url: String = if config.permalink.is_empty() {
                    format!("{}.html", config.title)
                } else {
                    config.permalink.clone() // messy.... argh!!!
                };
                let config_liquid = liquid::object!({
                    "content": template,
                    "config": liquid::object!({
                        "title": config.title,
                        "description": config.description,
                        "tags": config.tags,
                        "categories": config.categories,
                        "visible": config.visible,
                        "layout": config.layout
                    }),
                });
                // if let Some(base_layout) =
                return Ok(Article {
                    template,
                    config,
                    url,
                    config_liquid,
                });
            }
        } else {
            return Err(ParseError::NoHeader);
        }

        Err(ParseError::InvalidTemplate)
    }

    pub fn render(
        &self,
        globals: &liquid::Object,
        parser: &liquid::Parser,
    ) -> Result<String, CustomError> {
        let template = if self.config.base_layout.is_empty() {
            warn!("no base layout found");
            parser.parse(&self.template)?
        } else {
            warn!("using baselayout: {:?}", self.config.base_layout);
            parser.parse(&format!("{{%- include '{0}' -%}}", self.config.base_layout))?
        };

        Ok(template.render(&liquid::object!({
            "global": globals,
            "page": self.config_liquid,
            "layout": "page"
        }))?)
    }
}

// read_to_string
// use std::fs::read_to_string;

#[test]
fn test_empty_content() {
    assert_eq!(Some(ParseError::Empty), Article::parse("").err());
}

#[test]
fn test_empty_template() {
    let a: Article = Article::parse("---\nlayout:page\ntitle:cats and dogs---\n").unwrap();
    assert_eq!("", a.template);
}

#[test]
fn test_parse() {
    let a: Article = Article::parse("---\nlayout:page\ntitle:cats and dogs---\ncat").unwrap();
    assert_eq!("cat", a.template);
    assert_eq!("page", a.config.layout);
}

#[test]
fn test_parse_template_muli_line() {
    let a: Article =
        Article::parse("---\nlayout:page\ntitle:cats and dogs---\ncat\ncat\ncat\ncat\ncat")
            .unwrap();
    assert_eq!("cat\ncat\ncat\ncat\ncat", a.template);
    assert_eq!("page", a.config.layout);
}

#[test]
fn test_template_md_line() {
    let a: Article = Article::parse("---\nlayout:page\ntitle:cats and dogs---\ncat---dog").unwrap();
    assert_eq!("cat---dog", a.template);
    assert_eq!("page", a.config.layout);
}

#[test]
fn test_parse_with_real() {
    let a: Article =
        Article::parse("---\r\nlayout: page\r\ntitle:cats and dogs---\r\ncat").unwrap();
    assert_eq!("cat", a.template);
    assert_eq!("page", a.config.layout);
}

#[test]
fn test_more_than_three_dashes() {
    assert_eq!(
        Some(ParseError::InvalidHeader(
            "only 3 dashes allowed for config header".to_string()
        )),
        Article::parse("----\r\nlayout:page\r\ntitle:cats and dogs-------\r\ncat").err()
    );
}
