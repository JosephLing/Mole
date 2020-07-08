use std::borrow;

use liquid::ValueView;
#[derive(Default, Debug, Clone, Copy)]
struct TestFileSystem;

impl liquid::partials::PartialSource for TestFileSystem {
    fn contains(&self, _name: &str) -> bool {
        true
    }

    fn names(&self) -> Vec<&str> {
        vec![]
    }

    fn try_get<'a>(&'a self, name: &str) -> Option<borrow::Cow<'a, str>> {
        let template = match name {
            "product" => "Product: {{ product.title }} ".into(),
            "locale_variables" => "Locale: {{echo1}} {{echo2}}".into(),
            "variant" => "Variant: {{ variant.title }}".into(),
            "nested_template" => {
                "{% include 'header' %} {% include 'body' %} {% include 'footer' %}".into()
            }
            "body" => "body {% include 'body_detail' %}".into(),
            "nested_product_template" => {
                "Product: {{ nested_product_template.title }} {%include 'details'%} ".into()
            }
            "recursively_nested_template" => "-{% include 'recursively_nested_template' %}".into(),
            "pick_a_source" => "from TestFileSystem".into(),
            "assignments" => "{% assign foo = 'bar' %}".into(),
            _ => name.to_owned().into(),
        };
        Some(template)
    }
}
