// fn main() {
//     use pulldown_cmark::{html, Options, Parser};

//     let markdown_input = "{% include 'nested_template' %}{{head}}{{ 'tobi' | upcase }}Hello world, this is a ~~complicated~~ *very simple* example.";

//     // Set up options and parser. Strikethroughs are not part of the CommonMark standard
//     // and we therefore must enable it explicitly.
//     let mut options = Options::empty();
//     options.insert(Options::ENABLE_STRIKETHROUGH);
//     let parser = Parser::new_ext(markdown_input, options);

//     // Write to String buffer.
//     let mut html_output = String::new();
//     html::push_html(&mut html_output, parser);

//     // Check that the output is what we expected.
//     let expected_html =
//         "<p>{% include 'nested_template' %}{{head}}{{ 'tobi' | upcase }}Hello world, this is a <del>complicated</del> <em>very simple</em> example.</p>\n";
//     assert_eq!(expected_html, &html_output);
//     type Partials = liquid::partials::EagerCompiler<liquid::partials::InMemorySource>;
//     let parser = Partials::empty();
//     parser.add("body", "header {{content}} footer");
//     parser.add("post", "");
//     let template = liquid::ParserBuilder::with_stdlib()
//         .partials(liquid::partials::OnDemandCompiler::<TestFileSystem>::empty())
//         .build()
//         .unwrap()
//         .parse(&html_output)
//         .unwrap();

//     let output = template
//         .render(&liquid::object!({
//             "head": "cats",
//             "cat": 10,
//             // "header": template2
//         }))
//         .unwrap();

//     let expected_html =
//         "<p>header boy body_detailscatsTOBIHello world, this is a <del>complicated</del> <em>very simple</em> example.</p>\n";
//     assert_eq!(expected_html, &output);
// }

// include
// artilces - .md .js (parsing with babel or rust babel)
// place meta-data into liquid
// output

/* todo:

- paths
- include
- meta data
- output
*/
use argh::FromArgs;
use log::{info, warn};

pub mod cmds;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct TopLevel {
    #[argh(subcommand)]
    nested: cmds::SubCommands,
}



fn main() {
    simple_logger::init().unwrap();
    info!("such information");
    warn!("o_O");
    argh::from_env::<TopLevel>().nested.run();
}
// // run source output
