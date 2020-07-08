pub mod parse;
use pulldown_cmark::{html, Options, Parser};

#[derive(Debug, PartialEq)]
enum ContentType {
    Markdown,
}

fn parse_file(
    name: &str,
    content: &str,
    contentType: ContentType,
) -> Result<parse::Article, parse::ParseError> {
    let mut article = parse::Article::parse(content)?;

    if contentType == ContentType::Markdown {
        let parser = Parser::new_ext(&article.template, Options::empty());

        // Write to String buffer.
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        article.template = html_output;
        return Ok(article);
    }

    // hack of an error message
    return Err(parse::ParseError::InvalidTemplate);
}

#[test]
fn test_markdown() {
    let a: parse::Article = parse_file(
        "template.txt",
        "---\nlayout:page\ntitle:cats and dogs---\ncat",
        ContentType::Markdown,
    )
    .unwrap();
    assert_eq!("<p>cat</p>\n", a.template);
}
