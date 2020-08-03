use crate::error::CustomError;
use log::warn;
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;

type ErrorMessage = String;

#[derive(PartialEq)]
pub enum ParseError {
    InvalidKey(ErrorMessage),
    EmptyValue(ErrorMessage),
    InvalidValue(ErrorMessage),
    InvalidConfig(ErrorMessage),
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::InvalidKey(s) => write!(f, "Invalid key: {}\n", s),
            ParseError::EmptyValue(s) => write!(f, "Empty value\n{}\n", s),
            ParseError::InvalidValue(s) => write!(f, "Invalid value: {}\n", s),
            ParseError::InvalidConfig(s) => write!(f, "Invalid configuration: {}\n", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub layout: String,
    pub base_layout: String,
    pub title: String,
    pub description: String,
    pub permalink: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub visible: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            layout: String::from(""),
            base_layout: String::from(""),
            title: String::from(""),
            description: String::from(""),
            permalink: String::from(""),
            categories: Vec::new(),
            tags: Vec::new(),
            visible: false,
        }
    }
}

impl Config {
    fn is_valid(&self) -> bool {
        !(self.layout.is_empty() || self.title.is_empty())
    }
}

fn parse_error_message(
    message: &str,
    path: &str,
    line: &str,
    start: usize,
    end: usize,
    lineno: usize,
) -> ErrorMessage {
    let spacing = if lineno < 99 {
        "  "
    } else if lineno < 999 {
        "   "
    } else {
        "    "
    };

    let mut underline = String::new();
    for _i in 0..start {
        underline.push(' ');
    }

    for _i in start..end {
        underline.push('^');
    }

    let msg : ErrorMessage = format!(
        "\n{s   } --> {p} {n}:{start}\n{s   } |\n{n:w$} | {line}\n{s   } | {underline}\n{s   } |\n{s  }{m}",
        p = path,
        line = line,
        s = spacing,
        w = spacing.len(),
        underline = underline,
        n = lineno,
        start = start,
        m = message
    )
    .to_string();

    msg
}

fn parse_key<'a>(
    rest: &'a str,
    path: &str,
    line: &str,
    lineno: usize,
) -> Result<(&'a str, &'a str), ParseError> {
    if rest.is_empty() {
        return Err(ParseError::EmptyValue(parse_error_message(
            "expected name of key",
            path,
            line,
            line.len(),
            line.len() + 5,
            lineno,
        )));
    }
    if let Some(index) = rest.find(":") {
        return Ok((&rest[0..index], &rest[index + 1..]));
    }
    Err(ParseError::InvalidKey(parse_error_message(
        "no semicolon found",
        path,
        line,
        line.len(),
        line.len()+1,
        lineno,
    )))
}

fn parse_value_string<'a>(
    rest: &'a str,
    path: &str,
    line: &str,
    lineno: usize,
) -> Result<&'a str, ParseError> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Err(ParseError::EmptyValue(parse_error_message(
            "empty value",
            path,
            line,
            line.len(),
            line.len() + 5,
            lineno,
        )));
    }
    if rest == "---" {
        return Err(ParseError::InvalidValue(parse_error_message(
            "found '---' can't use configuration start and end identifier as a value",
            path,
            line,
            line.len() - 3,
            line.len(),
            lineno,
        )));
    }
    Ok(rest)
}

fn parse_value_boolean(
    rest: &str,
    path: &str,
    line: &str,
    lineno: usize,
) -> Result<bool, ParseError> {
    match rest.parse::<bool>() {
        Ok(b) => Ok(b),
        Err(_) => Err(ParseError::InvalidValue(parse_error_message(
            "",
            path,
            line,
            line.len() - rest.len(),
            line.len(),
            lineno,
        ))),
    }
}

fn parse_value_list(
    rest: &str,
    path: &str,
    line: &str,
    lineno: usize,
) -> Result<Vec<String>, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::EmptyValue(parse_error_message(
            "empty",
            path,
            line,
            line.len(),
            line.len() + 5,
            lineno,
        )));
    }
    let mut list: Vec<String> = Vec::new();
    let mut index = 0;
    let mut expect = false;
    while !rest[index..].is_empty() {
        // consume [ and ] but only if they are present
        if let Some(new_index) = rest[index..].find(",") {
            expect = true;
            list.push(parse_value_string(&rest[index..new_index], path, line, lineno)?.to_string());
            index = new_index + 1;
        } else {
            // if it is empty between index
            list.push(parse_value_string(&rest[index..], path, line, lineno)?.to_string());
            index = rest.len();
            expect = false;
        }
    }
    if expect {
        return Err(ParseError::InvalidValue(parse_error_message(
            "value expected after semi-colon",
            path,
            line,
            line.len(),
            line.len()+5,
            lineno,
        )));
    }
    Ok(list)
}

/// BufReader or read_to_string() is the key api choice (mmap alternatively as well)
/// the difficulty getting the rest of the file after parsing the config
/// BufReader<R> can improve the speed of programs that make small and repeated read calls to the same file or network socket.
/// It does not help when reading very large amounts at once, or reading just one or a few times.
/// It also provides no advantage when reading from a source that is already in memory, like a Vec<u8>.
pub fn parse(data: &str) -> Result<(Config, String), ParseError> {
    let mut found_config = false;
    let mut line_n = 1;
    let mut config = Config::default();
    let lines = data.lines();
    let mut body = "".to_string();
    let path = "test.txt";
    let mut reached_end = false;
    for line in lines {
        if !found_config && line == "---" {
            found_config = true;
        } else if found_config  && line == "---" {
            reached_end = true;
            found_config = false;
        }else if reached_end{
            body += line;
            body += "\n";
        } else if found_config{
            let (key, rest) = parse_key(&line, path, line, line_n)?;
            match key {
                // match each thing but then need to work out how to map it....
                // maybe look into the from string implementation???
                "layout" => {
                    config.layout = parse_value_string(rest.trim(), path, line, line_n)?.to_string()
                }
                "base_layout" => {
                    config.base_layout =
                        parse_value_string(rest.trim(), path, line, line_n)?.to_string()
                }
                "title" => {
                    config.title = parse_value_string(rest.trim(), path, line, line_n)?.to_string()
                }
                "description" => {
                    config.description = parse_value_string(rest.trim(), path, line, line_n)?.to_string()
                }
                "permalink" => {
                    config.permalink =
                        parse_value_string(rest.trim(), path, line, line_n)?.to_string()
                }
                "categories" => {
                    config.categories = parse_value_list(rest.trim(), path, line, line_n)?
                }
                "tags" => config.tags = parse_value_list(rest.trim(), path, line, line_n)?,
                "visible" => config.visible = parse_value_boolean(rest.trim(), path, line, line_n)?,
                _ => {
                    return Err(ParseError::InvalidKey(parse_error_message(
                        "unknown key",
                        path,
                        line,
                        0,
                        line.len()-1,
                        line_n,
                    )))
                }
            }
        }else{
            return Err(ParseError::InvalidConfig(parse_error_message("configuration needs to start with '---' for the first line", path, line, 0, line.len(), line_n)));
        }
        line_n += 1;
    }
    if config.is_valid() {
        return Ok((config, body));
    } else if line_n == 2 {
        return Err(ParseError::InvalidConfig(
            format!("empty config no key value pairs found in {}", "test.txt").into(),
        ));
    } else if !reached_end {
        return Err(ParseError::InvalidConfig(
            "no at '---' for the last line of the configuration".into(),
        ));
    } else if config.title.is_empty(){
        return Err(ParseError::InvalidConfig(
            "missing configuration 'title' field".into(),
        ));
    }else {
        return Err(ParseError::InvalidConfig(
            "missing configuration 'layout' field or 'base_layout' to be set to a custom value".into(),
        ));
    }
}


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
    pub fn parse(md: &str) -> Result<Article, ParseError> {
        let (config, content) = parse(md)?;        
        // markdown parsing NOTE: we are assuming that we are dealing with markdown hear!!!
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
        // if let Some(base_layout) =
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
            warn!("no base layout found");
            parser.parse(&self.template)?
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
mod parse_tests {

    use super::*;

    #[test]
    fn empty_content() {
        assert_eq!(Some(ParseError::InvalidConfig("no at '---' for the last line of the configuration".into())), Article::parse("").err());
    }

    #[test]
    fn test_empty_template() {
        let a: Article = Article::parse("---\nlayout:page\ntitle:cats and dogs\n---\n").unwrap();
        assert_eq!("", a.template);
    }

    #[test]
    fn parse() {
        let a: Article = Article::parse("---\nlayout:page\ntitle:cats and dogs\n---\ncat").unwrap();
        assert_eq!("cat", a.template);
        assert_eq!("page", a.config.layout);
    }

    #[test]
    fn parse_template_muli_line() {
        let a: Article =
            Article::parse("---\nlayout:page\ntitle:cats and dogs\n---\ncat\ncat\ncat\ncat\ncat")
                .unwrap();
        assert_eq!("cat\ncat\ncat\ncat\ncat", a.template);
        assert_eq!("page", a.config.layout);
    }

    #[test]
    fn template_md_line() {
        let a: Article =
            Article::parse("---\nlayout:page\ntitle:cats and dogs\n---\ncat---dog").unwrap();
        assert_eq!("cat---dog", a.template);
        assert_eq!("page", a.config.layout);
    }

    #[test]
    fn parse_with_real() {
        let a: Article =
            Article::parse("---\nlayout: page\ntitle:cats and dogs\n---\ncat").unwrap();
        assert_eq!("cat", a.template);
        assert_eq!("page", a.config.layout);
    }

    #[test]
    fn more_than_three_dashes() {
        assert_eq!(
            Some(ParseError::InvalidConfig(
                "only 3 dashes allowed for config header".into()
            )),
            Article::parse("----\nlayout:page\ntitle:cats and dogs\n-------\ncat").err()
        );
    }
}

#[cfg(test)]
mod pre_render {

    use super::*;

    // lazy didn't know how best to grab the type
    type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;

    fn gen_render_mocks(md: &str, mocks: Vec<(String, String)>) -> Result<String, CustomError> {
        let a: Article = Article::parse(md).unwrap();
        let mut source = Partials::empty();

        for (k, v) in mocks {
            source.add(k, v);
        }
        let parser = liquid::ParserBuilder::with_stdlib()
            .partials(source)
            .build()?;

        Ok(
            a.pre_render(&liquid::object!({ "articles": vec![md] }), &parser, false)
                .pre_render(&liquid::object!({ "articles": vec![md] }), &parser, true)
                .template,
        )
    }

    #[test]
    fn markdown_and_page_varaibles() {
        assert_eq!(
            "<h1>cats and dogs</h1>\n<p>cats</p>\n".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\n# {{page.config.title}}\ncats",
                vec![("default".to_string(), "cats".to_string())]
            )
            .unwrap()
        );
    }

    #[test]
    fn accessing_global_varaible() {
        assert_eq!(
            "<h1>cats and dogs</h1>\n<p>cats</p>\n".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\n# {{page.config.title}}\ncats",
                vec![("default".to_string(), "cats".to_string())]
            )
            .unwrap()
        );
    }

    // we want some way of stopping an article iterating over all the articles or at least itself
    #[ignore = "awaiting error reporting"]
    #[test]
    fn error_when_article_accesses_itself() {
        assert_eq!(
            Some(CustomError("".to_string())),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\n# {{global.articles}}\ncat",
                vec![("default".to_string(), "cats".to_string())]
            )
            .err()
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

        a.pre_render(&liquid::object!({}), &parser, true)
            .render(global, &parser)
    }

    #[test]
    fn render_default_layout() {
        assert_eq!(
            "cats".to_string(),
            gen_render_mocks(
                "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\ncat",
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
                "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\ncat",
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
                "---\r\nlayout: page\r\ntitle:cats and dogs\n---\r\ncat",
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
