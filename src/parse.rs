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
                        "layout": config.layout,
                    }),
                    "url":url,
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

    pub fn pre_render(mut self, liquidParser: &liquid::Parser) -> Self {
        // hack do proper error handling!!!

        println!("{:?}", self.config_liquid);
        let template = liquidParser
            .parse(&self.template)
            .unwrap()
            .render(&liquid::object!({
                "page": self.config_liquid
                // "page": liquid::object!({
                //     "title": "cats and dogs"
                // })
            }))
            .unwrap();

        let parser = Parser::new_ext(&template, Options::empty());

        // Write to String buffer.
        let mut template = String::new();
        html::push_html(&mut template, parser);

        self.template = template;

        self.config_liquid = liquid::object!({
            "content": self.template,
            "config": liquid::object!({
                "title": self.config.title,
                "description": self.config.description,
                "tags": self.config.tags,
                "categories": self.config.categories,
                "visible": self.config.visible,
                "layout": self.config.layout,
            }),
            "url":self.url,
        });
        self
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

#[cfg(test)]
mod parse {

    use super::*;

    #[test]
    fn empty_content() {
        assert_eq!(Some(ParseError::Empty), Article::parse("").err());
    }

    #[test]
    fn test_empty_template() {
        let a: Article = Article::parse("---\nlayout:page\ntitle:cats and dogs---\n").unwrap();
        assert_eq!("", a.template);
    }

    #[test]
    fn parse() {
        let a: Article = Article::parse("---\nlayout:page\ntitle:cats and dogs---\ncat").unwrap();
        assert_eq!("cat", a.template);
        assert_eq!("page", a.config.layout);
    }

    #[test]
    fn parse_template_muli_line() {
        let a: Article =
            Article::parse("---\nlayout:page\ntitle:cats and dogs---\ncat\ncat\ncat\ncat\ncat")
                .unwrap();
        assert_eq!("cat\ncat\ncat\ncat\ncat", a.template);
        assert_eq!("page", a.config.layout);
    }

    #[test]
    fn template_md_line() {
        let a: Article =
            Article::parse("---\nlayout:page\ntitle:cats and dogs---\ncat---dog").unwrap();
        assert_eq!("cat---dog", a.template);
        assert_eq!("page", a.config.layout);
    }

    #[test]
    fn parse_with_real() {
        let a: Article =
            Article::parse("---\r\nlayout: page\r\ntitle:cats and dogs---\r\ncat").unwrap();
        assert_eq!("cat", a.template);
        assert_eq!("page", a.config.layout);
    }

    #[test]
    fn more_than_three_dashes() {
        assert_eq!(
            Some(ParseError::InvalidHeader(
                "only 3 dashes allowed for config header".to_string()
            )),
            Article::parse("----\r\nlayout:page\r\ntitle:cats and dogs-------\r\ncat").err()
        );
    }
}


#[cfg(test)]
mod pre_render {

    use super::*;

    use super::*;

    // lazy didn't know how best to grab the type
    type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

    fn gen_render_mocks (
        md: &str,
        mocks: Vec<(String, String)>,
    ) -> Result<String, CustomError> {
        let a: Article = Article::parse(md).unwrap();
        let mut source = Partials::empty();

        for (k, v) in mocks {
            source.add(k, v);
        }
        let parser = liquid::ParserBuilder::with_stdlib()
            .partials(source)
            .build()?;

        Ok(a.pre_render(&parser).template)
    }

    #[test]
    fn markdown_and_page_varaibles() {
        assert_eq!(
            "<h1>cats and dogs</h1>\n<p>cats</p>\n".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs---\r\n# {{page.config.title}}\ncats",
                vec![("default".to_string(), "cats".to_string())]
            ).unwrap()
        );
    }

    #[test]
    fn error_when_accessing_global() {
        assert_eq!(
            Some(CustomError("".to_string())),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs---\r\n# {{global}}\ncat",
                vec![("default".to_string(), "cats".to_string())]
            ).err()
        );
    }

}



// read_to_string
// use std::fs::read_to_string;
#[cfg(test)]
mod render {
    use super::*;

    // lazy didn't know how best to grab the type
    type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

    fn gen_render_mocks(
        md: &str,
        mocks: Vec<(String, String)>,
        global: &liquid::Object,
    ) -> Result<String, CustomError> {
        let a: Article = Article::parse(md).unwrap();
        let mut source = Partials::empty();

        for (k, v) in mocks {
            source.add(k, v);
        }
        let parser = liquid::ParserBuilder::with_stdlib()
            .partials(source)
            .build()
            .unwrap();

        a.pre_render(&parser).render(global, &parser)
    }

    #[test]
    fn render_default_layout() {
        assert_eq!(
            "cats".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs---\r\ncat",
                vec![("default".to_string(), "cats".to_string())],
                &liquid::object!({})
            )
            .unwrap()
        );
    }

    #[test]
    fn render_globals() {
        assert_eq!(
            "test1".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs---\r\ncat",
                vec![("default".to_string(), "{{global}}".to_string())],
                &liquid::object!({
                    "test": 1
                })
            )
            .unwrap()
        );
    }

    #[test]
    fn render_globals_scope() {
        assert_eq!(
            "1".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs---\r\ncat",
                vec![("default".to_string(), "{{global.test}}".to_string())],
                &liquid::object!({
                    "test": 1
                })
            )
            .unwrap()
        );
    }

    #[test]
    fn render_content() {
        assert_eq!(
            "<h1>cats and dogs</h1><p>cat</p>\n".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs---\r\ncat",
                vec![(
                    "default".to_string(),
                    "<h1>{{page.config.title}}</h1>{{page.content}}".to_string()
                )],
                &liquid::object!({
                    "test": 1
                })
            )
            .unwrap()
        );
    }

    #[test]
    fn render_chained_includes() {
        assert_eq!(
            "I am a header<p>cat</p>\n".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs---\r\ncat",
                vec![
                    (
                        "default".to_string(),
                        "{% include 'header' %}{% include layout %}".to_string()
                    ),
                    ("header".to_string(), "I am a header".to_string()),
                    ("page2".to_string(), "1".to_string()),
                    ("page3".to_string(), "2".to_string()),
                    ("page".to_string(), "{{page.content}}".to_string()),
                    ("page4".to_string(), "3".to_string())
                ],
                &liquid::object!({
                    "test": 1
                })
            )
            .unwrap()
        );
    }

    #[test]
    fn render_template_jekyll() {
        assert_eq!(
            "<h1>mole</h1><p>cat mole</p>\n".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:mole---\r\ncat {{page.config.title}}",
                vec![
                    (
                        "default".to_string(),
                        "<h1>{{page.config.title}}</h1>{% include layout %}".to_string()
                    ),
                    ("page".to_string(), "{{page.content}}".to_string())
                ],
                &liquid::object!({
                    "test": 1
                })
            )
            .unwrap()
        );
    }
}
