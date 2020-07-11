use std::collections::HashMap;
use log::{info, warn, error};

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
    pub base_layout: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub permalink: Option<String>,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub visible: bool
}

/*


---
layout: page
title: Tags
permalink: /tags/
titlebar: false
---

*/

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
            let temp: Vec<&str> = line.split(":").collect();
            if let Some(key) = temp.get(0) {
                if let Some(value) = temp.get(1) {
                    pieces.insert(key.to_string(), value.to_string());
                }
            }
        }
        if let Some(layout) = pieces.get("layout") {
            if let Some(title) = pieces.get("title") {
                
                // duplicate code ugh...
                // one potential solution would be to use serde_yaml but that's yaml....
                // these could be matches
                let mut desc: Option<String> = None;
                if let Some(description) = pieces.get("description") {
                    desc = Some(description.to_string());
                }

                let mut perma: Option<String> = None;
                if let Some(permalink) = pieces.get("permalink") {
                    perma = Some(permalink.to_string());
                }

                // let mut vis: bool = true;
                // if let Some(visible) = pieces.get("visible") {
                //     if let Ok(temp) = visible.parse::<bool>(){
                //         vis = temp;
                //     }
                // }


                Ok(Config {
                    // TODO: how do we handle spaces at the start of layout and title e.g. layout: page won't match "page" as it would be " page"
                    layout: layout.to_string(),
                    title: title.to_string(),
                    description: desc,
                    // TODO: read in base_layout as an option
                    base_layout: Some("default".to_string()),
                    permalink: perma,
                    // TODO: these as lists would be really nice.... potentailly thinking about spinning out the json example again maybe....
                    categories: None,
                    tags: None,
                    visible: match pieces.get("visible") {
                        Some(b) => match b.parse::<bool>(){
                            Ok(b) => b,
                            Err(_) => true,
                        },
                        None => true
                    }
                })
            } else {
                return Err(ParseError::InvalidHeader(String::from("title error")));
            }
        } else {
            return Err(ParseError::InvalidHeader(String::from("layout error")));
        }
    }
}

#[test]
fn test_config_layout() {
    assert_eq!(
        Some(ParseError::InvalidHeader(String::from("layout error"))),
        Config::parse("").err()
    );
}

#[test]
fn test_config_title() {
    assert_eq!(
        Some(ParseError::InvalidHeader(String::from("title error"))),
        Config::parse("layout: page").err()
    );
}

#[test]
fn test_config() {
    assert_eq!(
        Config {
            layout: "page".to_string(),
            base_layout: Some("default".to_string()), 
            title: "hello world".to_string(),
            description: None,
            permalink: None,
            categories: None,
            tags: None,
            visible: true
        },
        Config::parse("layout:page\ntitle:hello world").unwrap()
    );
}

#[derive(Debug)]
pub struct Article {
    pub template: String,
    pub config: Config,
}

impl Article {
    /// header is in a --- --- block with new lines
    /// the rest of the doc is template in markdown
    pub fn parse(md: &str) -> Result<Article, ParseError> {
        info!("parsing article");
        info!("{:?}", md);
        info!("{:?}", md.trim());
        if md.is_empty() {
            return Err(ParseError::Empty);
        }

        let lines: Vec<&str> = md.split("---\r\n").collect();
        if let Some(config) = lines.get(1) {
            if let Some(content) = lines.get(2) {
                info!("parsing article - config headers found");
                return Ok(Article {
                    template: content.to_string(),
                    config: Config::parse(config)?,
                });
            }
        } else {
            return Err(ParseError::NoHeader);
        }

        return Err(ParseError::InvalidTemplate);
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
fn test_parse_with_real() {
    let a: Article = Article::parse("---\r\nlayout:page\r\ntitle:cats and dogs---\r\ncat").unwrap();
    assert_eq!("cat", a.template);
    assert_eq!("page", a.config.layout);
}