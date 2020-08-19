use crate::parse::{parse, Config, ParseError};

use crate::error::CustomError;
use log::warn;
use pulldown_cmark::{html, Options, Parser};
use std::{fs::File, io::BufReader};

#[derive(Debug)]
pub struct Article {
    pub template: String,
    pub config: Config,
    pub url: String,
    pub config_liquid: liquid::Object,
}

impl Article {
    /// header is in a --- --- block with new lines
    /// the rest of the doc is template in markdown
    pub fn parse(md: BufReader<File>, path: &str) -> Result<Article, ParseError> {
        // markdown parsing NOTE: we are assuming that we are dealing with markdown hear!!!
        let (config, content) = parse(md, path)?;

        let template = content.trim().to_string();

        let url: String = if config.permalink.is_empty() {
            format!("{}.html", config.title)
        } else {
            config.permalink.clone() // messy.... argh!!!
        }
        .replace(" ", "%20");

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

        return Ok(Article {
            template,
            config,
            url,
            config_liquid,
        });
    }

    pub fn pre_render(
        mut self,
        globals: &liquid::Object,
        liquidParser: &liquid::Parser,
        md: bool,
    ) -> Self {
        // hack do proper error handling!!!

        let template = liquidParser
            .parse(&self.template)
            .unwrap()
            .render(&liquid::object!({
                "global": globals,
                "page": self.config_liquid,
                "layout": self.config.layout
            }))
            .unwrap();

        self.template = if md {
            let parser = Parser::new_ext(&template, Options::empty());

            // Write to String buffer.
            let mut template = String::new();
            html::push_html(&mut template, parser);
            template
        } else {
            template
        };

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
            if self.config.layout.is_empty() {
                warn!("no base layout found");
                parser.parse(&format!("{{%- include '{0}' -%}}", self.config.layout))?
            } else {
                warn!("no base layout found and no layout found");
                parser.parse(&self.template)?
            }
        } else {
            warn!("using baselayout: {:?}", self.config.base_layout);
            parser.parse(&format!("{{%- include '{0}' -%}}", self.config.base_layout))?
        };

        Ok(template.render(&liquid::object!({
            "global": globals,
            "page": self.config_liquid,
            "layout": self.config.layout
        }))?)
    }
}

#[cfg(test)]
mod render {

    use super::*;
    use std::io::Write;
    use tempfile;

    // lazy didn't know how best to grab the type
    type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

    fn create_article(md: &str, path: &str) -> Result<Article, ParseError> {
        // create a temp file
        let mut f = tempfile::Builder::new()
            .rand_bytes(0)
            .prefix("")
            .suffix(path)
            .tempfile_in("")
            .unwrap();
        write!(f, "{}", md).unwrap();

        Ok(Article::parse(
            BufReader::new(File::open(path).unwrap()),
            path,
        )?)
    }

    fn gen_render_mocks(
        md: &str,
        path: &str,
        mocks: Vec<(String, String)>,
        global: &liquid::Object,
    ) -> Result<String, CustomError> {
        let a = create_article(md, path).unwrap();
        // create partials
        let mut source = Partials::empty();
        for (k, v) in mocks {
            source.add(k, v);
        }
        let parser = liquid::ParserBuilder::with_stdlib()
            .partials(source)
            .build()
            .unwrap();

        a.pre_render(global, &parser, true).render(global, &parser)
    }

    mod parse_tests {

        use super::*;

        #[test]
        fn empty_content() {
            assert_eq!(
                Some(ParseError::InvalidConfig(
                    "no at '---' for the last line of the configuration".into()
                )),
                create_article("", "empty_content").err()
            );
        }

        #[test]
        fn test_empty_template() {
            let a: Article =
                create_article("---\nlayout:page\ntitle:cats and dogs\n---\n", "test_empty_template").unwrap();
            assert_eq!("", a.template);
        }

        #[test]
        fn parse() {
            let a: Article =
            create_article("---\nlayout:page\ntitle:cats and dogs\n---\ncat", "parse").unwrap();
            assert_eq!("cat", a.template);
            assert_eq!("page", a.config.layout);
        }

        #[test]
        fn parse_template_muli_line() {
            let a: Article = create_article(
                "---\nlayout:page\ntitle:cats and dogs\n---\ncat\ncat\ncat\ncat\ncat", "parse_template_multi_line"
            )
            .unwrap();
            assert_eq!("cat\ncat\ncat\ncat\ncat", a.template);
            assert_eq!("page", a.config.layout);
        }

        #[test]
        fn template_md_line() {
            let a: Article =
            create_article("---\nlayout:page\ntitle:cats and dogs\n---\ncat---dog", "template_md_line").unwrap();
            assert_eq!("cat---dog", a.template);
            assert_eq!("page", a.config.layout);
        }

        #[test]
        fn parse_with_real() {
            let a: Article =
            create_article("---\nlayout: page\ntitle:cats and dogs\n---\ncat", "parse_with_real").unwrap();
            assert_eq!("cat", a.template);
            assert_eq!("page", a.config.layout);
        }

        #[test]
        fn more_than_three_dashes() {
            assert_eq!(
                Some(ParseError::InvalidConfig(
                    "only 3 dashes allowed for config header".into()
                )),
                create_article("----\nlayout:page\ntitle:cats and dogs\n-------\ncat", "more_than_three_seconds").err()
            );
        }
    }

    mod layouts {
        use super::*;

        #[test]
        fn render_default_layout() {
            assert_eq!(
                "<p>catscat</p>\n",
                gen_render_mocks(
                    "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\ncat",
                    "render_default_layout",
                    vec![("default".to_string(), "cats".to_string())],
                    &liquid::object!({})
                )
                .unwrap()
            );
        }

        #[test]
        fn render_globals() {
            assert_eq!(
                "test1cat",
                gen_render_mocks(
                    "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\ncat",
                    "render_globals",
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
                    "render_globals_scope",
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
                    "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\ncat",
                    "render_content",
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
        fn render_content_with_html_in_md() {
            assert_eq!(
                "<h1>cats and dogs</h1><p>cat<span>hello world</span></p>\n".to_string(),
                gen_render_mocks(
                    "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\ncat<span>hello world</span>",
                    "render_content_with_html_in_md",
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
                    "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\ncat",
                    "render_chained_includes",
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
                    "---\r\nlayout: page\r\ntitle:mole\n---\r\ncat {{page.config.title}}",
                    "render_template_jekyll",
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
}
