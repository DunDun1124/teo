use std::fmt::{Debug};
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{Date, DateTime, Utc};
use crate::core::argument::Argument::ValueArgument;
use crate::core::pipeline::Pipeline;
use crate::core::stage::Stage;
use crate::core::value::Value;
use crate::core::object::Object;


#[async_trait]
pub trait FnArgument: Debug + Send + Sync {
    fn name(&self) -> String;
    async fn call(&self, value: Value, object: Object) -> Stage;
}

#[derive(Debug, Clone)]
pub enum Argument {
    ValueArgument(Value),
    PipelineArgument(Pipeline),
    FunctionArgument(Arc<dyn FnArgument>),
}

impl From<&str> for Argument {
    fn from(v: &str) -> Self {
        ValueArgument(Value::String(v.to_string()))
    }
}

impl From<String> for Argument {
    fn from(v: String) -> Self {
        ValueArgument(Value::String(v))
    }
}

impl From<bool> for Argument {
    fn from(v: bool) -> Self {
        ValueArgument(Value::Bool(v))
    }
}

impl From<i8> for Argument {
    fn from(v: i8) -> Self {
        ValueArgument(Value::I8(v))
    }
}

impl From<i16> for Argument {
    fn from(v: i16) -> Self {
        ValueArgument(Value::I16(v))
    }
}

impl From<i32> for Argument {
    fn from(v: i32) -> Self {
        ValueArgument(Value::I32(v))
    }
}

impl From<i64> for Argument {
    fn from(v: i64) -> Self {
        ValueArgument(Value::I64(v))
    }
}

impl From<i128> for Argument {
    fn from(v: i128) -> Self {
        ValueArgument(Value::I128(v))
    }
}

impl From<u8> for Argument {
    fn from(v: u8) -> Self {
        ValueArgument(Value::U8(v))
    }
}

impl From<u16> for Argument {
    fn from(v: u16) -> Self {
        ValueArgument(Value::U16(v))
    }
}

impl From<u32> for Argument {
    fn from(v: u32) -> Self {
        ValueArgument(Value::U32(v))
    }
}

impl From<u64> for Argument {
    fn from(v: u64) -> Self {
        ValueArgument(Value::U64(v))
    }
}

impl From<u128> for Argument {
    fn from(v: u128) -> Self {
        ValueArgument(Value::U128(v))
    }
}

impl From<f32> for Argument {
    fn from(v: f32) -> Self {
        ValueArgument(Value::F32(v))
    }
}

impl From<f64> for Argument {
    fn from(v: f64) -> Self {
        ValueArgument(Value::F64(v))
    }
}

impl From<Date<Utc>> for Argument {
    fn from(v: Date<Utc>) -> Self {
        ValueArgument(Value::Date(v))
    }
}

impl From<DateTime<Utc>> for Argument {
    fn from(v: DateTime<Utc>) -> Self {
        ValueArgument(Value::DateTime(v))
    }
}

// impl<T: Into<Argument>> From<Vec<T>> for Argument {
//     fn from(v: Vec<T>) -> Self {
//         ValueArgument(Value::Vec(v.iter().map(|i| {
//             let value: Argument
//         }).collect()))
//     }
// }
