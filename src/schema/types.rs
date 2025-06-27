use serde_json::Value;
use std::collections::HashSet;

pub fn detect_string_format(s: &str) -> Option<&'static str> {
    if s.contains('@') && s.contains('.') {
        Some("email")
    } else if s.starts_with("http") {
        Some("uri")
    } else if s.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ' ') {
        None
    } else {
        None
    }
}

pub fn detect_string_pattern(s: &str) -> Option<&'static str> {
    if s.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ' ') {
        Some(r"^[\d\-\s]+$")
    } else {
        None
    }
}

pub fn get_array_item_types(arr: &[Value]) -> HashSet<&'static str> {
    arr.iter()
        .map(|v| match v {
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Null => "null",
        })
        .collect()
}

pub fn is_homogeneous_array(arr: &[Value]) -> bool {
    get_array_item_types(arr).len() <= 1
}