use kstring::KString;
use liquid::Object;
use log::warn;
use std::io::Write;

use liquid_core::Error;
use liquid_core::Expression;
use liquid_core::Language;
use liquid_core::Renderable;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::ValueView;
use liquid_core::{error::ResultLiquidExt, Value};
use liquid_core::{ParseTag, TagReflection, TagTokenIter};

#[derive(Debug)]
struct Include {
    partial: Expression,
    vars: Vec<(KString, Expression)>,
}

impl Renderable for Include {
    fn render_to(&self, writer: &mut dyn Write, runtime: &mut Runtime<'_>) -> Result<()> {
        let value = self.partial.evaluate(runtime)?;
        if !value.is_scalar() {
            return Error::with_msg("Can only `include` strings")
                .context("partial", format!("{}", value.source()))
                .into_err();
        }

        let name = value.to_kstr().into_owned();
        runtime.run_in_named_scope(name.clone(), |mut scope| -> Result<()> {
            // if there our additional varaibles creates a include object to access all the varaibles
            // from e.g. { include 'image.html' path="foo.png" }
            // then in image.html you could have <img src="{{include.path}}" />
            if !self.vars.is_empty() {
                let mut helper_vars = Object::new();

                for (id, val) in &self.vars {
                    helper_vars.insert(
                        id.clone(),
                        val.try_evaluate(scope)
                            .ok_or_else(|| Error::with_msg("failed to evaluate value"))?
                            .into_owned(),
                    );
                }

                scope.stack_mut().set("include", Value::Object(helper_vars));
            }

            let partial = scope
                .partials()
                .get(&name)
                .trace_with(|| format!("{{% include {} %}}", self.partial).into())?;

            partial
                .render_to(writer, &mut scope)
                .trace_with(|| format!("{{% include {} %}}", self.partial).into())
                .context_key_with(|| self.partial.to_string().into())
                .value_with(|| name.to_string().into())
        })?;

        Ok(())
    }

    fn render(&self, runtime: &mut Runtime<'_>) -> Result<String> {
        let mut data = Vec::new();
        self.render_to(&mut data, runtime)?;
        Ok(String::from_utf8(data).expect("render only writes UTF-8"))
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct IncludeTag;

impl TagReflection for IncludeTag {
    fn tag(&self) -> &'static str {
        "include"
    }

    fn description(&self) -> &'static str {
        ""
    }
}

impl ParseTag for IncludeTag {
    fn parse(
        &self,
        mut arguments: TagTokenIter<'_>,
        _options: &Language,
    ) -> Result<Box<dyn Renderable>> {
        let name = arguments.expect_next("Identifier or literal expected.")?;

        let name = name.expect_value().into_result()?;
        match name.clone() {
            Expression::Variable(v) => {
                warn!(
                    "potential jekyll include tag found: {0} (fix add '{0}')",
                    v.to_string()
                        .replace("[", ".")
                        .replace("]", "")
                        .replace("\"", "")
                );
            }
            Expression::Literal(_) => {}
        }

        let mut vars: Vec<(KString, Expression)> = Vec::new();
        while let Ok(next) = arguments.expect_next("") {
            let id = next.expect_value().into_result()?.to_string();

            arguments
                .expect_next("expected string")?
                .expect_str("=")
                .into_result_custom_msg("expected '=' to be used for the assignment")?;

            vars.push((
                id.into(),
                arguments
                    .expect_next("expected expression/value")?
                    .expect_value()
                    .into_result()?,
            ));
        }

        arguments.expect_nothing()?;

        Ok(Box::new(Include {
            partial: name,
            vars,
        }))
    }

    fn reflection(&self) -> &dyn TagReflection {
        self
    }
}
