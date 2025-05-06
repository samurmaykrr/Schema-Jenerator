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

    #[test_case("simple.json", "--pretty"; "pretty formatting")]
    #[test_case("simple.json", "--strict"; "strict mode")]
    #[test_case("simple.json", "--assert-values"; "value assertions")]
    fn test_cli_options(file_name: &str, option: &str) {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join(file_name);
        let output_path = dir.path().join(format!("{}.schema.json", file_name.split(".").next().unwrap()));

        fs::write(&input_path, r#"{"name": "test", "value": 42}"#).unwrap();

        let mut cmd = Command::cargo_bin("schema-jenerator").unwrap();

        cmd.arg(&input_path)
            .arg(option)
            .assert()
            .success();

        assert!(output_path.exists());

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains(r#""type": "object"#));
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
}
