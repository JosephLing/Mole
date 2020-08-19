use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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
            base_layout: String::from("default"),
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
    lineno: i8,
) -> ErrorMessage {
    let spacing = if lineno < 99 {
        "  "
    } else if lineno < 127 {
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
    lineno: i8,
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
        line.len() + 1,
        lineno,
    )))
}

fn parse_value_string<'a>(
    rest: &'a str,
    path: &str,
    line: &str,
    lineno: i8,
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

fn parse_value_boolean(rest: &str, path: &str, line: &str, lineno: i8) -> Result<bool, ParseError> {
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
    lineno: i8,
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
            line.len() + 5,
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
pub fn parse(data: BufReader<File>, path: &str) -> Result<(Config, String), ParseError> {
    let mut found_config = false;
    let mut line_n = 1;
    let mut config = Config::default();
    // we set the defaults here e.g. default_layout: "default"
    // therefore when we get default_layout: "" then it overwrites the default
    let lines = data.lines();
    let mut body = "".to_string();
    let mut reached_end = false;
    for line in lines {
        let line = &line.unwrap();
        if !found_config && line == "---" {
            found_config = true;
            line_n += 1;
        } else if found_config && line == "---" {
            reached_end = true;
            found_config = false;
            line_n += 1;
        } else if reached_end {
            body += &line;
            body += "\n";
        } else if found_config {
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
                    config.description =
                        parse_value_string(rest.trim(), path, line, line_n)?.to_string()
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
                        line.len() - 1,
                        line_n,
                    )))
                }
            }
            line_n += 1;
        } else {
            return Err(ParseError::InvalidConfig(parse_error_message(
                "configuration needs to start with '---' for the first line",
                path,
                line,
                0,
                line.len(),
                line_n,
            )));
        }
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
    } else if config.title.is_empty() {
        return Err(ParseError::InvalidConfig(
            "missing configuration 'title' field".into(),
        ));
    } else {
        return Err(ParseError::InvalidConfig(
            "missing configuration 'layout' field or 'base_layout' to be set to a custom value"
                .into(),
        ));
    }
}
