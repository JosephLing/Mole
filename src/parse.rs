use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::path::PathBuf;

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

pub fn parse_error_message(
    message: &str,
    path: &PathBuf,
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
        p = path.to_str().unwrap(),
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

pub fn parse_key<'a>(
    rest: &'a str,
    path: &PathBuf,
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

pub fn parse_value_string<'a>(
    rest: &'a str,
    path: &PathBuf,
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

    if rest.starts_with('"') {
        if !rest.ends_with('"') {
            return Err(ParseError::InvalidValue(parse_error_message(
                "string started with \" character but did not close string at the end",
                path,
                line,
                0,
                line.len(),
                lineno,
            )));
        } else {
            return Ok(&rest[1..rest.len() - 1]);
        }
    }

    if rest.starts_with("'") {
        if !rest.ends_with("'") {
            return Err(ParseError::InvalidValue(parse_error_message(
                "string started with \" character but did not close string at the end",
                path,
                line,
                0,
                line.len(),
                lineno,
            )));
        } else {
            return Ok(&rest[1..rest.len() - 1]);
        }
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

pub fn parse_value_boolean(
    rest: &str,
    path: &PathBuf,
    line: &str,
    lineno: i8,
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

pub fn parse_value_time(
    rest: &str,
    path: &PathBuf,
    line: &str,
    lineno: i8,
) -> Result<NaiveDateTime, ParseError> {
    match NaiveDate::parse_from_str(rest, "%Y-%m-%d") {
        Ok(date) => Ok(date.and_time(NaiveTime::from_hms_milli(0, 0, 0, 0))),
        Err(_) => match NaiveDateTime::parse_from_str(rest, "%Y-%m-%d %H:%M") {
            Ok(date) => Ok(date),
            Err(err) => Err(ParseError::InvalidValue(parse_error_message(
                &("date error: ".to_owned() + &err.to_string() + " expected Y-m-d or Y-m-d h:m"),
                path,
                line,
                line.len() - rest.len(),
                line.len(),
                lineno,
            ))),
        },
    }
}

pub fn parse_value_list(
    mut rest: &str,
    path: &PathBuf,
    line: &str,
    lineno: i8,
) -> Result<Vec<String>, ParseError> {
    rest = rest.trim();
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
    let mut prev = 0;
    let mut in_string = false;
    let mut in_string_lower = false;

    if rest.starts_with("[") {
        if rest.ends_with("]") {
            rest = rest.trim_start_matches("[").trim_end_matches("]");
        } else {
            return Err(ParseError::InvalidValue(parse_error_message(
                "found opening square bracket for list but no opening bracket",
                path,
                line,
                0,
                line.len(),
                lineno,
            )));
        }
    }

    let bytes = rest.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b',' && !in_string && !in_string_lower {
            list.push(parse_value_string(&rest[prev..i], path, line, lineno)?.to_string());
            prev = i + 1;
        } else if item == b'"' && !in_string_lower {
            in_string = !in_string;
        } else if item == b'\'' && !in_string {
            in_string_lower = !in_string_lower;
        }
    }
    if prev == rest.len() {
        return Err(ParseError::InvalidValue(parse_error_message(
            "value expected after semi-colon",
            path,
            line,
            line.len(),
            line.len() + 5,
            lineno,
        )));
    } else if in_string {
        return Err(ParseError::InvalidValue(parse_error_message(
            "found a string but no closing \"",
            path,
            line,
            line.len() - 1,
            line.len(),
            lineno,
        )));
    } else if in_string_lower {
        return Err(ParseError::InvalidValue(parse_error_message(
            "found a string but no closing \'",
            path,
            line,
            line.len() - 1,
            line.len(),
            lineno,
        )));
    } else {
        list.push(parse_value_string(&rest[prev..], path, line, lineno)?.to_string());
    }

    Ok(list)
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_key_test() {
        let line = "hello: world";
        let (key, rest) = parse_key(line, &PathBuf::from("test.txt"), line, 1).unwrap();
        assert_eq!(key, "hello");
        assert_eq!(rest, " world");
    }

    #[test]
    fn parse_key_no_semicolon() {
        let line = "hello  world";
        let err = parse_key(line, &PathBuf::from("test.txt"), line, 1).err();
        match err {
            Some(ParseError::InvalidKey(config)) => assert!(
                config.contains("no semicolon found"),
                "expected 'no semicolon found' in {}",
                config
            ),
            _ => assert!(false, "expected error"),
        }
    }

    #[test]
    fn parse_value_list_multi_spaced() {
        let line = "a, b, c, d";
        let list = parse_value_list(line, &PathBuf::from("test.txt"), line, 1).unwrap();
        assert_eq!(vec!["a", "b", "c", "d"], list);
    }

    #[test]
    fn parse_value_list_single() {
        let line = "a";
        let list = parse_value_list(line, &PathBuf::from("test.txt"), line, 1).unwrap();
        assert_eq!(vec!["a"], list);
    }

    #[test]
    fn parse_value_list_double_no_spaced() {
        let line = "a, b";
        let list = parse_value_list(line, &PathBuf::from("test.txt"), line, 1).unwrap();
        assert_eq!(vec!["a", "b"], list);
    }

    #[test]
    fn parse_value_list_square_brackets() {
        let line = "[a, b]";
        let list = parse_value_list(line, &PathBuf::from("test.txt"), line, 1).unwrap();
        assert_eq!(vec!["a", "b"], list);
    }

    #[test]
    fn parse_value_list_sauare_brackets_err() {
        let line = "[a, b";
        let err = parse_value_list(line, &PathBuf::from("test.txt"), line, 1).err();
        match err {
            Some(ParseError::InvalidValue(config)) => assert!(
                config.contains("found opening square bracket for list but no opening bracket"),
                "found opening square bracket for list but no opening bracket' in {}",
                config
            ),
            _ => assert!(false, "expected error"),
        }
    }

    #[test]
    fn parse_value_list_single_quote() {
        let line = "',a', 'b'";
        let list = parse_value_list(line, &PathBuf::from("test.txt"), line, 1).unwrap();
        assert_eq!(vec![",a", "b"], list);
    }

    #[test]
    fn parse_value_list_double_quote() {
        let line = "\",a\", \"b\"";
        let list = parse_value_list(line, &PathBuf::from("test.txt"), line, 1).unwrap();
        assert_eq!(vec![",a", "b"], list);
    }

    #[test]
    fn parse_value_list_err() {
        let line = "a, b,";
        let err = parse_value_list(line, &PathBuf::from("test.txt"), line, 1).err();
        match err {
            Some(ParseError::InvalidValue(config)) => assert!(
                config.contains("value expected after semi-colon"),
                "expected 'value expected after semi-colon' in {}",
                config
            ),
            _ => assert!(false, "expected error"),
        }
    }
}
