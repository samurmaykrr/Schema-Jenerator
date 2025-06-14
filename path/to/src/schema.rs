use serde_json::{Value, Map};
use crate::schema::generate_schema;

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
    }

    fn test_generate_array_schema() {
        // Homogeneous array
        let homogeneous = json!([1, 2, 3, 4]);
        let homo_schema = generate_schema(&homogeneous, false, false).unwrap();
        assert_eq!(homo_schema["type"], "array");
        assert_eq!(homo_schema["items"]["type"], "integer");

        // Heterogeneous array
        let heterogeneous = json!([1, "string", true]);
        let hetero_schema = generate_schema(&heterogeneous, false, false).unwrap();
    }

    fn test_generate_primitive_schemas() {
        // String
        let string = json!("test");
        let string_schema = generate_schema(&string, false, false).unwrap();
        assert_eq!(string_schema["type"], "string");

        // Number
        let number = json!(42.5);
        let number_schema = generate_schema(&number, false, false).unwrap();
        assert_eq!(number_schema["type"], "number");

    }

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
            }
        });
    }
}
