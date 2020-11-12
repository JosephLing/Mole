use liquid::ValueView;
use liquid_core::{to_value, Display_filter, Filter, FilterReflection, ParseFilter};
use log::debug;

#[derive(Clone, FilterReflection)]
#[filter(
    name = "to_json",
    description = "Serialize a value to json",
    parsed(ToJsonFilter)
)]
pub struct ToJson;

#[derive(Debug, Display_filter)]
#[name = "to_json"]
struct ToJsonFilter {}

impl ParseFilter for ToJson {
    fn parse(
        &self,
        _arguments: liquid_core::parser::FilterArguments,
    ) -> liquid_core::Result<Box<dyn Filter>> {
        Ok(Box::new(ToJsonFilter {}))
    }

    fn reflection(&self) -> &dyn FilterReflection {
        self as &dyn FilterReflection
    }
}

impl Filter for ToJsonFilter {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &liquid_core::Runtime,
    ) -> liquid_core::Result<liquid_core::Value> {
        let output = serde_json::to_string_pretty(&input.to_value()).unwrap();
        // debug!("to_json: {}",output);
        Ok(to_value(&output)?)
    }
}
