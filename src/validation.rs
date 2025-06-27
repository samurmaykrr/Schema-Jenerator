use jsonschema::JSONSchema;
use serde_json::Value;
use anyhow::Result;

use crate::error::AppError;

pub fn validate_schema(schema: &Value) -> Result<()> {
    let meta_schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema"
    });
    
    match JSONSchema::compile(&meta_schema) {
        Ok(compiled) => {
            if let Err(errors) = compiled.validate(schema) {
                let error_messages: Vec<String> = errors
                    .map(|e| e.to_string())
                    .collect();
                return Err(AppError::SchemaGeneration(
                    format!("Schema validation failed: {}", error_messages.join(", "))
                ).into());
            }
            println!("Schema validation passed");
        }
        Err(e) => {
            return Err(AppError::SchemaGeneration(
                format!("Failed to compile meta-schema: {}", e)
            ).into());
        }
    }
    
    Ok(())
}

pub fn validate_json_against_schema(json: &Value, schema: &Value) -> Result<()> {
    match JSONSchema::compile(schema) {
        Ok(compiled) => {
            if let Err(errors) = compiled.validate(json) {
                let error_messages: Vec<String> = errors
                    .map(|e| e.to_string())
                    .collect();
                return Err(AppError::SchemaGeneration(
                    format!("JSON validation failed: {}", error_messages.join(", "))
                ).into());
            }
            Ok(())
        }
        Err(e) => {
            Err(AppError::SchemaGeneration(
                format!("Failed to compile schema for validation: {}", e)
            ).into())
        }
    }
}