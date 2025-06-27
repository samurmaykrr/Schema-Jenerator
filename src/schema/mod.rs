use serde_json::Value;
use clap::ValueEnum;
use anyhow::Result;

pub mod generators;
pub mod types;

pub use generators::*;
pub use types::*;

#[derive(Debug, Clone, ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum SchemaOutputTier {
    Basic,
    Standard,
    Comprehensive,
    Expert,
}

pub fn generate_schema(value: &Value, tier: &SchemaOutputTier) -> Result<Value> {
    match value {
        Value::Object(obj) => generate_object_schema(obj, tier),
        Value::Array(arr) => generate_array_schema(arr, tier),
        Value::String(_) => generate_string_schema(value, tier),
        Value::Number(n) => generate_number_schema(n, tier),
        Value::Bool(_) => generate_boolean_schema(value, tier),
        Value::Null => generate_null_schema(),
    }
}