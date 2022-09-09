use async_trait::async_trait;
use serde_json::{Value as JsonValue};
use crate::core::json_pipeline::context::JsonPipelineContext;
use crate::core::json_pipeline::json_path::json_set;
use crate::core::json_pipeline::JsonPipelineItem;
use crate::core::pipeline::stage::Stage;

#[derive(Debug)]
pub(crate) struct SetItem {
    key: String,
    value: JsonValue,
}

impl SetItem {
    pub(crate) fn new(key: impl Into<String>, value: JsonValue) -> Self {
        Self { key: key.into(), value }
    }
}

#[async_trait]
impl JsonPipelineItem for SetItem {
    async fn call(&self, context: JsonPipelineContext) -> JsonPipelineContext {
        let mut new_value = context.value().cloned();
        let new_value = if let Some(new_value) = new_value {
            Some(json_set(&new_value, vec![self.key.clone()], self.value.clone()))
        } else {
            None
        };
        JsonPipelineContext::construct(new_value, context.location().clone(), context.object().clone(), context.stage(), context.identity().cloned())
    }
}