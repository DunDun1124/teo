use async_trait::async_trait;

use crate::core::pipeline::modifier::Modifier;
use crate::core::teon::Value;
use crate::core::pipeline::context::Context;

#[derive(Debug, Clone)]
pub struct RegexMatchModifier {
    argument: Value
}

impl RegexMatchModifier {
    pub fn new(format: impl Into<Value>) -> Self {
        Self {
            argument: format.into()
        }
    }
}

#[async_trait]
impl Modifier for RegexMatchModifier {

    fn name(&self) -> &'static str {
        "regexMatch"
    }

    async fn call<'a>(&self, ctx: Context<'a>) -> Context<'a> {
        let arg_value = self.argument.resolve(ctx.clone()).await;
        let regex = arg_value.as_regexp().unwrap();
        match &ctx.value {
            Value::String(s) => {
                if regex.is_match(s) {
                    ctx.clone()
                } else {
                    ctx.invalid(format!("Value does not match '{regex}'"))
                }
            }
            _ => {
                ctx.invalid("Value is not string.")
            }
        }
    }
}
