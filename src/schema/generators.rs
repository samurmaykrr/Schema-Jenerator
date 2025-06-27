use serde_json::{Value, Map};
use anyhow::Result;

use crate::schema::{SchemaOutputTier, types::*};

pub fn generate_object_schema(
    obj: &Map<String, Value>,
    tier: &SchemaOutputTier
) -> Result<Value> {
    let mut schema = serde_json::json!({
        "type": "object",
        "properties": {}
    });

    if matches!(tier, SchemaOutputTier::Comprehensive | SchemaOutputTier::Expert) {
        schema["$schema"] = Value::String("https://json-schema.org/draft/2020-12/schema".to_string());
    }

    let mut required_props = Vec::new();
    let mut properties_map = Map::new();
    
    for (key, value) in obj {
        let property_schema = crate::schema::generate_schema(value, tier)?;
        properties_map.insert(key.clone(), property_schema);

        match tier {
            SchemaOutputTier::Basic => {},
            SchemaOutputTier::Standard => {
                if !value.is_null() {
                    required_props.push(Value::String(key.clone()));
                }
            }
            SchemaOutputTier::Comprehensive | SchemaOutputTier::Expert => {
                required_props.push(Value::String(key.clone()));
            }
        }
    }

    schema["properties"] = Value::Object(properties_map);

    if !required_props.is_empty() {
        schema["required"] = Value::Array(required_props);
    }

    match tier {
        SchemaOutputTier::Basic => {},
        SchemaOutputTier::Standard => {
            schema["additionalProperties"] = Value::Bool(true);
        }
        SchemaOutputTier::Comprehensive => {
            schema["additionalProperties"] = Value::Bool(false);
            schema["minProperties"] = Value::Number(1.into());
        }
        SchemaOutputTier::Expert => {
            schema["additionalProperties"] = Value::Bool(false);
            schema["minProperties"] = Value::Number(1.into());
            schema["title"] = Value::String("Generated Object Schema".to_string());
            schema["description"] = Value::String("Auto-generated schema from JSON data".to_string());
        }
    }

    Ok(schema)
}

pub fn generate_array_schema(
    arr: &[Value],
    tier: &SchemaOutputTier
) -> Result<Value> {
    if arr.is_empty() {
        return Ok(serde_json::json!({
            "type": "array",
            "items": {}
        }));
    }

    let mut schema = serde_json::json!({
        "type": "array"
    });

    if is_homogeneous_array(arr) {
        let item_schema = crate::schema::generate_schema(&arr[0], tier)?;
        schema["items"] = item_schema;
    } else {
        let mut item_schemas = Vec::new();
        for item in arr {
            let item_schema = crate::schema::generate_schema(item, tier)?;
            item_schemas.push(item_schema);
        }
        schema["items"] = serde_json::json!({ "oneOf": item_schemas });
    }

    match tier {
        SchemaOutputTier::Basic => {},
        SchemaOutputTier::Standard => {
            if !arr.is_empty() {
                schema["minItems"] = Value::Number(0.into());
            }
        }
        SchemaOutputTier::Comprehensive => {
            if !arr.is_empty() {
                schema["minItems"] = Value::Number(1.into());
                schema["maxItems"] = Value::Number((arr.len() * 2).into());
            }
        }
        SchemaOutputTier::Expert => {
            if !arr.is_empty() {
                schema["minItems"] = Value::Number(1.into());
                schema["maxItems"] = Value::Number((arr.len() * 2).into());
                schema["uniqueItems"] = Value::Bool(true);
                schema["title"] = Value::String("Generated Array Schema".to_string());
                schema["description"] = Value::String("Auto-generated array schema from JSON data".to_string());
            }
        }
    }

    Ok(schema)
}

pub fn generate_string_schema(value: &Value, tier: &SchemaOutputTier) -> Result<Value> {
    let mut schema = serde_json::json!({ "type": "string" });

    if let Value::String(s) = value {
        match tier {
            SchemaOutputTier::Basic => {},
            SchemaOutputTier::Standard => {
                schema["minLength"] = Value::Number(0.into());
            }
            SchemaOutputTier::Comprehensive => {
                schema["minLength"] = Value::Number(0.into());
                schema["maxLength"] = Value::Number((s.len() * 2).into());
                if !s.is_empty() {
                    schema["examples"] = serde_json::json!([s]);
                }
            }
            SchemaOutputTier::Expert => {
                schema["minLength"] = Value::Number(0.into());
                schema["maxLength"] = Value::Number((s.len() * 2).into());
                if !s.is_empty() {
                    schema["examples"] = serde_json::json!([s]);
                    
                    if let Some(format) = detect_string_format(s) {
                        schema["format"] = Value::String(format.to_string());
                    } else if let Some(pattern) = detect_string_pattern(s) {
                        schema["pattern"] = Value::String(pattern.to_string());
                    }
                }
                schema["title"] = Value::String("Generated String Schema".to_string());
            }
        }
    }

    Ok(schema)
}

pub fn generate_number_schema(n: &serde_json::Number, tier: &SchemaOutputTier) -> Result<Value> {
    let mut schema = serde_json::json!({});

    if n.is_i64() || n.is_u64() {
        schema["type"] = Value::String("integer".into());
    } else {
        schema["type"] = Value::String("number".into());
    }

    match tier {
        SchemaOutputTier::Basic => {},
        SchemaOutputTier::Standard => {
            if let Some(n_val) = n.as_i64() {
                schema["minimum"] = serde_json::json!(n_val);
            } else if let Some(n_val) = n.as_f64() {
                schema["minimum"] = serde_json::json!(n_val);
            }
        }
        SchemaOutputTier::Comprehensive => {
            if let Some(n_val) = n.as_i64() {
                schema["examples"] = serde_json::json!([n_val]);
                schema["minimum"] = serde_json::json!(n_val - 1000);
                schema["maximum"] = serde_json::json!(n_val + 1000);
            } else if let Some(n_val) = n.as_f64() {
                schema["examples"] = serde_json::json!([n_val]);
                schema["minimum"] = serde_json::json!(n_val - 1000.0);
                schema["maximum"] = serde_json::json!(n_val + 1000.0);
            }
        }
        SchemaOutputTier::Expert => {
            if let Some(n_val) = n.as_i64() {
                schema["examples"] = serde_json::json!([n_val]);
                schema["minimum"] = serde_json::json!(n_val - 1000);
                schema["maximum"] = serde_json::json!(n_val + 1000);
                schema["multipleOf"] = serde_json::json!(1);
                schema["title"] = Value::String("Generated Integer Schema".to_string());
            } else if let Some(n_val) = n.as_f64() {
                schema["examples"] = serde_json::json!([n_val]);
                schema["minimum"] = serde_json::json!(n_val - 1000.0);
                schema["maximum"] = serde_json::json!(n_val + 1000.0);
                schema["title"] = Value::String("Generated Number Schema".to_string());
            }
        }
    }

    Ok(schema)
}

pub fn generate_boolean_schema(value: &Value, tier: &SchemaOutputTier) -> Result<Value> {
    let mut schema = serde_json::json!({ "type": "boolean" });

    if let Value::Bool(b) = value {
        match tier {
            SchemaOutputTier::Basic | SchemaOutputTier::Standard => {},
            SchemaOutputTier::Comprehensive => {
                schema["examples"] = serde_json::json!([b]);
            }
            SchemaOutputTier::Expert => {
                schema["examples"] = serde_json::json!([b]);
                schema["title"] = Value::String("Generated Boolean Schema".to_string());
                schema["description"] = Value::String("Boolean value from JSON data".to_string());
            }
        }
    }

    Ok(schema)
}

pub fn generate_null_schema() -> Result<Value> {
    Ok(serde_json::json!({ "type": "null" }))
}