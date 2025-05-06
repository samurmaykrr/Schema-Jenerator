
use serde_json::{Value, Map};
use std::collections::HashSet;
use anyhow::Result;
use log::debug;

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

        // Test with default options
        let schema = generate_schema(&input, false, false).unwrap();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert_eq!(schema["properties"]["name"]["type"], "string");
        assert_eq!(schema["properties"]["age"]["type"], "integer");
        assert_eq!(schema["properties"]["is_active"]["type"], "boolean");

        // Test with strict mode
        let strict_schema = generate_schema(&input, true, false).unwrap();
        assert_eq!(strict_schema["additionalProperties"], false);
        assert!(strict_schema["required"].is_array());
        assert!(strict_schema["required"].as_array().unwrap().contains(&json!("name")));

        // Test with value assertions
        let assert_schema = generate_schema(&input, false, true).unwrap();
        assert!(assert_schema["properties"]["name"]["examples"].is_array());
        assert_eq!(assert_schema["properties"]["name"]["examples"][0], "John");
    }

    #[test]
    fn test_generate_array_schema() {
        // Homogeneous array
        let homogeneous = json!([1, 2, 3, 4]);
        let homo_schema = generate_schema(&homogeneous, false, false).unwrap();
        assert_eq!(homo_schema["type"], "array");
        assert_eq!(homo_schema["items"]["type"], "integer");

        // Heterogeneous array
        let heterogeneous = json!([1, "string", true]);
        let hetero_schema = generate_schema(&heterogeneous, false, false).unwrap();
        assert_eq!(hetero_schema["type"], "array");
        assert!(hetero_schema["items"]["oneOf"].is_array());
        assert_eq!(hetero_schema["items"]["oneOf"].as_array().unwrap().len(), 3);

        // Empty array
        let empty = json!([]);
        let empty_schema = generate_schema(&empty, false, false).unwrap();
        assert_eq!(empty_schema["type"], "array");
        assert!(empty_schema["items"].is_object());
        assert!(empty_schema["items"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_generate_primitive_schemas() {
        // String
        let string = json!("test");
        let string_schema = generate_schema(&string, false, false).unwrap();
        assert_eq!(string_schema["type"], "string");

        // Number
        let number = json!(42.5);
        let number_schema = generate_schema(&number, false, false).unwrap();
        assert_eq!(number_schema["type"], "number");

        // Integer
        let integer = json!(42);
        let integer_schema = generate_schema(&integer, false, false).unwrap();
        assert_eq!(integer_schema["type"], "integer");

        // Boolean
        let boolean = json!(true);
        let boolean_schema = generate_schema(&boolean, false, false).unwrap();
        assert_eq!(boolean_schema["type"], "boolean");

        // Null
        let null = json!(null);
        let null_schema = generate_schema(&null, false, false).unwrap();
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

        let schema = generate_schema(&complex, true, true).unwrap();

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


/// Generate a JSON schema based on the given JSON value
pub fn generate_schema(
    value: &Value,
    strict: bool,
    assert_values: bool
) -> Result<Value> {
    match value {
        Value::Object(obj) => generate_object_schema(obj, strict, assert_values),
        Value::Array(arr) => generate_array_schema(arr, strict, assert_values),
        Value::String(_) => generate_string_schema(value, assert_values),
        Value::Number(n) => generate_number_schema(n, assert_values),
        Value::Bool(_) => generate_boolean_schema(value, assert_values),
        Value::Null => generate_null_schema(),
    }
}

fn generate_object_schema(
    obj: &Map<String, Value>,
    strict: bool,
    assert_values: bool
) -> Result<Value> {
    let mut schema = serde_json::json!({
        "type": "object",
        "properties": {},
        "required": [],
    });

    // Create a Vec to hold required property names
    let mut required_props = Vec::new();

    // First pass: generate schemas for properties
    let mut properties_map = Map::new();
    for (key, value) in obj {
        let property_schema = generate_schema(value, strict, assert_values)?;
        properties_map.insert(key.clone(), property_schema);

        // In strict mode, all properties are required
        if strict {
            required_props.push(Value::String(key.clone()));
        }
    }

    // Now update the schema values
    schema["properties"] = Value::Object(properties_map);

    if !required_props.is_empty() {
        schema["required"] = Value::Array(required_props);
    }

    if strict {
        // In strict mode, don't allow additional properties
        schema["additionalProperties"] = Value::Bool(false);
    }

    Ok(schema)
}

fn generate_array_schema(
    arr: &[Value],
    strict: bool,
    assert_values: bool
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
        let item_schema = generate_schema(&arr[0], strict, assert_values)?;
        schema["items"] = item_schema;
    } else {
        // Heterogeneous array
        let mut item_schemas = Vec::new();
        for item in arr {
            let item_schema = generate_schema(item, strict, assert_values)?;
            item_schemas.push(item_schema);
        }
        schema["items"] = serde_json::json!({ "oneOf": item_schemas });
    }

    if strict && !arr.is_empty() {
        // In strict mode, enforce minimum items
        schema["minItems"] = Value::Number(arr.len().into());
    }

    Ok(schema)
}

fn generate_string_schema(value: &Value, assert_values: bool) -> Result<Value> {
    let mut schema = serde_json::json!({ "type": "string" });

    if assert_values {
        if let Value::String(s) = value {
            schema["minLength"] = Value::Number(0.into());

            // Optional: Add pattern or enum for specific values
            if !s.is_empty() {
                schema["examples"] = serde_json::json!([s]);
            }
        }
    }

    Ok(schema)
}

fn generate_number_schema(n: &serde_json::Number, assert_values: bool) -> Result<Value> {
    let mut schema = serde_json::json!({});

    // Determine if it's an integer or float
    if n.is_i64() || n.is_u64() {
        schema["type"] = Value::String("integer".into());
    } else {
        schema["type"] = Value::String("number".into());
    }

    if assert_values {
        if let Some(n_val) = n.as_i64() {
            schema["examples"] = serde_json::json!([n_val]);
        } else if let Some(n_val) = n.as_f64() {
            schema["examples"] = serde_json::json!([n_val]);
        }
    }

    Ok(schema)
}

fn generate_boolean_schema(value: &Value, assert_values: bool) -> Result<Value> {
    let mut schema = serde_json::json!({ "type": "boolean" });

    if assert_values {
        if let Value::Bool(b) = value {
            schema["examples"] = serde_json::json!([b]);
        }
    }

    Ok(schema)
}

fn generate_null_schema() -> Result<Value> {
    Ok(serde_json::json!({ "type": "null" }))
}
