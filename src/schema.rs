
use serde_json::{Value, Map};
use std::collections::HashSet;
use anyhow::Result;
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum SchemaOutputTier {
    Basic,
    Standard,
    Comprehensive,
    Expert,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_object_schema() {
        let input = json!({
            "name": "John",
            "age": 30,
            "is_active": true
        });

        // Test with basic tier
        let schema = generate_schema(&input, &SchemaOutputTier::Basic).unwrap();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert_eq!(schema["properties"]["name"]["type"], "string");
        assert_eq!(schema["properties"]["age"]["type"], "integer");
        assert_eq!(schema["properties"]["is_active"]["type"], "boolean");

        // Test with comprehensive tier
        let comprehensive_schema = generate_schema(&input, &SchemaOutputTier::Comprehensive).unwrap();
        assert_eq!(comprehensive_schema["additionalProperties"], false);
        assert!(comprehensive_schema["required"].is_array());
        assert!(comprehensive_schema["required"].as_array().unwrap().contains(&json!("name")));

        // Test with expert tier
        let expert_schema = generate_schema(&input, &SchemaOutputTier::Expert).unwrap();
        assert!(expert_schema["properties"]["name"]["examples"].is_array());
        assert_eq!(expert_schema["properties"]["name"]["examples"][0], "John");
    }

    #[test]
    fn test_generate_array_schema() {
        // Homogeneous array
        let homogeneous = json!([1, 2, 3, 4]);
        let homo_schema = generate_schema(&homogeneous, &SchemaOutputTier::Standard).unwrap();
        assert_eq!(homo_schema["type"], "array");
        assert_eq!(homo_schema["items"]["type"], "integer");

        // Heterogeneous array
        let heterogeneous = json!([1, "string", true]);
        let hetero_schema = generate_schema(&heterogeneous, &SchemaOutputTier::Standard).unwrap();
        assert_eq!(hetero_schema["type"], "array");
        assert!(hetero_schema["items"]["oneOf"].is_array());
        assert_eq!(hetero_schema["items"]["oneOf"].as_array().unwrap().len(), 3);

        // Empty array
        let empty = json!([]);
        let empty_schema = generate_schema(&empty, &SchemaOutputTier::Standard).unwrap();
        assert_eq!(empty_schema["type"], "array");
        assert!(empty_schema["items"].is_object());
        assert!(empty_schema["items"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_generate_primitive_schemas() {
        // String
        let string = json!("test");
        let string_schema = generate_schema(&string, &SchemaOutputTier::Basic).unwrap();
        assert_eq!(string_schema["type"], "string");

        // Number
        let number = json!(42.5);
        let number_schema = generate_schema(&number, &SchemaOutputTier::Basic).unwrap();
        assert_eq!(number_schema["type"], "number");

        // Integer
        let integer = json!(42);
        let integer_schema = generate_schema(&integer, &SchemaOutputTier::Basic).unwrap();
        assert_eq!(integer_schema["type"], "integer");

        // Boolean
        let boolean = json!(true);
        let boolean_schema = generate_schema(&boolean, &SchemaOutputTier::Basic).unwrap();
        assert_eq!(boolean_schema["type"], "boolean");

        // Null
        let null = json!(null);
        let null_schema = generate_schema(&null, &SchemaOutputTier::Basic).unwrap();
        assert_eq!(null_schema["type"], "null");
    }

    #[test]
    fn test_complex_nested_structure() {
        let complex = json!({
            "person": {
                "name": "John",
                "contacts": [
                    {
                        "type": "email",
                        "value": "john@example.com"
                    },
                    {
                        "type": "phone",
                        "value": "123-456-7890"
                    }
                ]
            },
            "active": true,
            "scores": [95, 87, 92]
        });

        let schema = generate_schema(&complex, &SchemaOutputTier::Expert).unwrap();

        // Check top level properties
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["person"].is_object());
        assert_eq!(schema["properties"]["active"]["type"], "boolean");

        // Check nested object
        assert_eq!(schema["properties"]["person"]["type"], "object");
        assert_eq!(schema["properties"]["person"]["properties"]["name"]["type"], "string");

        // Check arrays
        assert_eq!(schema["properties"]["scores"]["type"], "array");
        assert_eq!(schema["properties"]["scores"]["items"]["type"], "integer");

        // Check array of objects
        assert_eq!(schema["properties"]["person"]["properties"]["contacts"]["type"], "array");
        assert_eq!(
            schema["properties"]["person"]["properties"]["contacts"]["items"]["type"],
            "object"
        );
    }
}


/// Generate a JSON schema based on the given JSON value and tier
pub fn generate_schema(
    value: &Value,
    tier: &SchemaOutputTier
) -> Result<Value> {
    match value {
        Value::Object(obj) => generate_object_schema(obj, tier),
        Value::Array(arr) => generate_array_schema(arr, tier),
        Value::String(_) => generate_string_schema(value, tier),
        Value::Number(n) => generate_number_schema(n, tier),
        Value::Bool(_) => generate_boolean_schema(value, tier),
        Value::Null => generate_null_schema(),
    }
}

fn generate_object_schema(
    obj: &Map<String, Value>,
    tier: &SchemaOutputTier
) -> Result<Value> {
    let mut schema = serde_json::json!({
        "type": "object",
        "properties": {}
    });

    // Add schema version for comprehensive and expert tiers
    if matches!(tier, SchemaOutputTier::Comprehensive | SchemaOutputTier::Expert) {
        schema["$schema"] = Value::String("https://json-schema.org/draft/2020-12/schema".to_string());
    }

    // Create a Vec to hold required property names
    let mut required_props = Vec::new();

    // First pass: generate schemas for properties
    let mut properties_map = Map::new();
    for (key, value) in obj {
        let property_schema = generate_schema(value, tier)?;
        properties_map.insert(key.clone(), property_schema);

        // Different tier behaviors for required properties
        match tier {
            SchemaOutputTier::Basic => {
                // Basic tier: no required properties
            }
            SchemaOutputTier::Standard => {
                // Standard tier: make non-null values required
                if !value.is_null() {
                    required_props.push(Value::String(key.clone()));
                }
            }
            SchemaOutputTier::Comprehensive | SchemaOutputTier::Expert => {
                // Comprehensive/Expert: all properties are required
                required_props.push(Value::String(key.clone()));
            }
        }
    }

    // Update schema properties
    schema["properties"] = Value::Object(properties_map);

    if !required_props.is_empty() {
        schema["required"] = Value::Array(required_props);
    }

    // Tier-specific enhancements
    match tier {
        SchemaOutputTier::Basic => {
            // Basic: minimal schema
        }
        SchemaOutputTier::Standard => {
            // Standard: some restrictions
            schema["additionalProperties"] = Value::Bool(true);
        }
        SchemaOutputTier::Comprehensive => {
            // Comprehensive: strict validation
            schema["additionalProperties"] = Value::Bool(false);
            schema["minProperties"] = Value::Number(1.into());
        }
        SchemaOutputTier::Expert => {
            // Expert: maximum validation and metadata
            schema["additionalProperties"] = Value::Bool(false);
            schema["minProperties"] = Value::Number(1.into());
            schema["title"] = Value::String("Generated Object Schema".to_string());
            schema["description"] = Value::String("Auto-generated schema from JSON data".to_string());
        }
    }

    Ok(schema)
}

fn generate_array_schema(
    arr: &[Value],
    tier: &SchemaOutputTier
) -> Result<Value> {
    if arr.is_empty() {
        return Ok(serde_json::json!({
            "type": "array",
            "items": {}
        }));
    }

    // For homogeneous arrays, we can use a single schema for all items
    // For heterogeneous arrays, we need to use oneOf with multiple schemas
    let item_types: HashSet<&str> = arr.iter()
        .map(|v| match v {
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Null => "null",
        })
        .collect();

    let mut schema = serde_json::json!({
        "type": "array"
    });

    if item_types.len() == 1 {
        // Homogeneous array
        let item_schema = generate_schema(&arr[0], tier)?;
        schema["items"] = item_schema;
    } else {
        // Heterogeneous array
        let mut item_schemas = Vec::new();
        for item in arr {
            let item_schema = generate_schema(item, tier)?;
            item_schemas.push(item_schema);
        }
        schema["items"] = serde_json::json!({ "oneOf": item_schemas });
    }

    // Tier-specific array constraints
    match tier {
        SchemaOutputTier::Basic => {
            // Basic: minimal constraints
        }
        SchemaOutputTier::Standard => {
            // Standard: some constraints
            if !arr.is_empty() {
                schema["minItems"] = Value::Number(0.into());
            }
        }
        SchemaOutputTier::Comprehensive => {
            // Comprehensive: strict constraints
            if !arr.is_empty() {
                schema["minItems"] = Value::Number(1.into());
                schema["maxItems"] = Value::Number((arr.len() * 2).into());
            }
        }
        SchemaOutputTier::Expert => {
            // Expert: maximum constraints and metadata
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

fn generate_string_schema(value: &Value, tier: &SchemaOutputTier) -> Result<Value> {
    let mut schema = serde_json::json!({ "type": "string" });

    if let Value::String(s) = value {
        match tier {
            SchemaOutputTier::Basic => {
                // Basic: just the type
            }
            SchemaOutputTier::Standard => {
                // Standard: basic constraints
                schema["minLength"] = Value::Number(0.into());
            }
            SchemaOutputTier::Comprehensive => {
                // Comprehensive: detailed constraints
                schema["minLength"] = Value::Number(0.into());
                schema["maxLength"] = Value::Number((s.len() * 2).into());
                if !s.is_empty() {
                    schema["examples"] = serde_json::json!([s]);
                }
            }
            SchemaOutputTier::Expert => {
                // Expert: maximum constraints and metadata
                schema["minLength"] = Value::Number(0.into());
                schema["maxLength"] = Value::Number((s.len() * 2).into());
                if !s.is_empty() {
                    schema["examples"] = serde_json::json!([s]);
                    // Detect common string patterns
                    if s.contains('@') && s.contains('.') {
                        schema["format"] = Value::String("email".to_string());
                    } else if s.starts_with("http") {
                        schema["format"] = Value::String("uri".to_string());
                    } else if s.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ' ') {
                        schema["pattern"] = Value::String(r"^[\d\-\s]+$".to_string());
                    }
                }
                schema["title"] = Value::String("Generated String Schema".to_string());
            }
        }
    }

    Ok(schema)
}

fn generate_number_schema(n: &serde_json::Number, tier: &SchemaOutputTier) -> Result<Value> {
    let mut schema = serde_json::json!({});

    // Determine if it's an integer or float
    if n.is_i64() || n.is_u64() {
        schema["type"] = Value::String("integer".into());
    } else {
        schema["type"] = Value::String("number".into());
    }

    match tier {
        SchemaOutputTier::Basic => {
            // Basic: just the type
        }
        SchemaOutputTier::Standard => {
            // Standard: basic constraints
            if let Some(n_val) = n.as_i64() {
                schema["minimum"] = serde_json::json!(n_val);
            } else if let Some(n_val) = n.as_f64() {
                schema["minimum"] = serde_json::json!(n_val);
            }
        }
        SchemaOutputTier::Comprehensive => {
            // Comprehensive: detailed constraints
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
            // Expert: maximum constraints and metadata
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

fn generate_boolean_schema(value: &Value, tier: &SchemaOutputTier) -> Result<Value> {
    let mut schema = serde_json::json!({ "type": "boolean" });

    if let Value::Bool(b) = value {
        match tier {
            SchemaOutputTier::Basic | SchemaOutputTier::Standard => {
                // Basic/Standard: just the type
            }
            SchemaOutputTier::Comprehensive => {
                // Comprehensive: add examples
                schema["examples"] = serde_json::json!([b]);
            }
            SchemaOutputTier::Expert => {
                // Expert: maximum metadata
                schema["examples"] = serde_json::json!([b]);
                schema["title"] = Value::String("Generated Boolean Schema".to_string());
                schema["description"] = Value::String("Boolean value from JSON data".to_string());
            }
        }
    }

    Ok(schema)
}

fn generate_null_schema() -> Result<Value> {
    Ok(serde_json::json!({ "type": "null" }))
}
