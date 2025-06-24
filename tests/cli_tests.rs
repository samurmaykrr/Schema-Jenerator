#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use tempfile::tempdir;
    use test_case::test_case;

    #[test]
    fn test_file_not_found() {
        let mut cmd = Command::cargo_bin("schema-jenerator").unwrap();

        cmd.arg("nonexistent_file.json")
            .assert()
            .failure()
            .stderr(predicate::str::contains("File not found"));
    }

    #[test]
    fn test_invalid_json() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("invalid.json");

        fs::write(&input_path, "{invalid: json}").unwrap();

        let mut cmd = Command::cargo_bin("schema-jenerator").unwrap();

        cmd.arg(input_path)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Invalid JSON"));
    }

    #[test_case("simple.json", "--pretty", None; "pretty formatting")]
    #[test_case("simple.json", "--tier", Some("comprehensive"); "comprehensive tier")]
    #[test_case("simple.json", "--tier", Some("expert"); "expert tier")]
    fn test_cli_options(file_name: &str, option1: &str, option2: Option<&str>) {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join(file_name);
        let output_path = dir.path().join(format!("{}.schema.json", file_name.split(".").next().unwrap()));

        fs::write(&input_path, r#"{"name": "test", "value": 42}"#).unwrap();

        let mut cmd = Command::cargo_bin("schema-jenerator").unwrap();

        cmd.arg(&input_path)
            .arg(option1);
        
        if let Some(option2) = option2 {
            cmd.arg(option2);
        }
        
        cmd.assert()
            .success();

        assert!(output_path.exists());

        let content = fs::read_to_string(output_path).unwrap();
        // Pretty formatting adds spaces, so we should be more flexible
        assert!(content.contains(r#""type""#) && content.contains(r#""object""#));
    }

    #[test]
    fn test_custom_output() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("input.json");
        let output_path = dir.path().join("custom_output.json");

        fs::write(&input_path, r#"{"name": "test", "value": 42}"#).unwrap();

        let mut cmd = Command::cargo_bin("schema-jenerator").unwrap();

        cmd.arg(&input_path)
            .arg("-o")
            .arg(&output_path)
            .assert()
            .success();

        assert!(output_path.exists());
    }

    #[test]
    fn test_schema_validation_with_sample_data() {
        let test_samples = [
            ("user_profile.json", include_str!("../test_samples/user_profile.json")),
            ("product_catalog.json", include_str!("../test_samples/product_catalog.json")),
            ("api_response.json", include_str!("../test_samples/api_response.json")),
        ];

        for (filename, content) in test_samples.iter() {
            let dir = tempdir().unwrap();
            let input_path = dir.path().join(filename);
            let output_path = dir.path().join(format!("{}.schema.json", filename.split('.').next().unwrap()));

            fs::write(&input_path, content).unwrap();

            // Test schema generation with validation
            let mut cmd = Command::cargo_bin("schema-jenerator").unwrap();
            cmd.arg(&input_path)
                .arg("--validate")
                .arg("--tier")
                .arg("expert")
                .assert()
                .success()
                .stdout(predicate::str::contains("Schema generated successfully"))
                .stdout(predicate::str::contains("Schema validation passed"));

            // Verify schema file was created
            assert!(output_path.exists(), "Schema file should exist for {}", filename);

            // Verify the generated schema is valid JSON
            let schema_content = fs::read_to_string(&output_path).unwrap();
            let schema_json: serde_json::Value = serde_json::from_str(&schema_content)
                .expect(&format!("Generated schema should be valid JSON for {}", filename));

            // Basic schema structure validation
            assert_eq!(schema_json["type"], "object", "Root should be object type for {}", filename);
            assert!(schema_json["properties"].is_object(), "Properties should be object for {}", filename);
            assert!(schema_json["$schema"].is_string(), "Schema should have $schema field for {}", filename);
        }
    }

    #[test]
    fn test_batch_processing_with_validation() {
        let dir = tempdir().unwrap();
        
        // Create multiple test files
        let test_files = [
            ("batch1.json", r#"{"name": "test1", "id": 1}"#),
            ("batch2.json", r#"{"name": "test2", "id": 2, "active": true}"#),
            ("batch3.json", r#"{"users": [{"name": "user1"}, {"name": "user2"}]}"#),
        ];

        for (filename, content) in test_files.iter() {
            let file_path = dir.path().join(filename);
            fs::write(&file_path, content).unwrap();
        }

        // Test batch processing with validation
        let pattern = dir.path().join("*.json");
        let mut cmd = Command::cargo_bin("schema-jenerator").unwrap();
        
        cmd.arg(pattern)
            .arg("--batch")
            .arg("--validate")
            .arg("--tier")
            .arg("comprehensive")
            .assert()
            .success()
            .stdout(predicate::str::contains("Processed 3 files successfully"));

        // Verify all schema files were created
        for (filename, _) in test_files.iter() {
            let schema_filename = format!("{}.schema.json", filename.split('.').next().unwrap());
            let schema_path = dir.path().join(schema_filename);
            assert!(schema_path.exists(), "Schema file should exist for {}", filename);
        }
    }
}
