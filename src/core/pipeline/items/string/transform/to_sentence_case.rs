use async_trait::async_trait;

use crate::core::pipeline::item::Item;
use crate::core::teon::Value;
use crate::core::result::Result;
use crate::core::pipeline::ctx::Ctx;
use inflector::Inflector;

#[derive(Debug, Copy, Clone)]
pub struct ToSentenceCaseItem {}

impl ToSentenceCaseItem {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Item for ToSentenceCaseItem {
    async fn call<'a>(&self, ctx: Ctx<'a>) -> Result<Ctx<'a>> {
        match ctx.get_value() {
            Value::String(s) => Ok(ctx.with_value(Value::String(s.to_sentence_case()))),
            _ => Err(ctx.internal_server_error("value is not string"))
        }
    }
}
