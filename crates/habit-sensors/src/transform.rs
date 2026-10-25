use habit_core::Finding;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TransformError {
    #[error("failed to execute jq transform: {message}")]
    ProcessFailed { message: String },

    #[error("transform output is not valid JSON: {message}")]
    InvalidJson { message: String },

    #[error("transform output must be a single JSON array, but received: {received}")]
    NotAnArray { received: String },

    #[error("transform output failed finding schema validation: {message}")]
    SchemaMismatch { message: String },
}

pub fn evaluate_jq(
    transform_path: &Path,
    input_json: &str,
) -> Result<Vec<Finding>, TransformError> {
    // Check if input is empty
    let trimmed_input = input_json.trim();
    if trimmed_input.is_empty() {
        return Ok(Vec::new());
    }

    let mut child = Command::new("jq")
        .arg("-f")
        .arg(transform_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| TransformError::ProcessFailed {
            message: format!("cannot spawn jq: {err}"),
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(trimmed_input.as_bytes())
            .map_err(|err| TransformError::ProcessFailed {
                message: format!("failed writing to jq stdin: {err}"),
            })?;
    }

    let output = child
        .wait_with_output()
        .map_err(|err| TransformError::ProcessFailed {
            message: format!("failed reading jq output: {err}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TransformError::ProcessFailed {
            message: stderr.to_string(),
        });
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let trimmed_stdout = stdout_str.trim();
    if trimmed_stdout.is_empty() {
        return Ok(Vec::new());
    }

    let parsed_val: serde_json::Value =
        serde_json::from_str(trimmed_stdout).map_err(|err| TransformError::InvalidJson {
            message: format!("{err}: output was {trimmed_stdout}"),
        })?;

    if !parsed_val.is_array() {
        return Err(TransformError::NotAnArray {
            received: trimmed_stdout.chars().take(80).collect(),
        });
    }

    let findings: Vec<Finding> =
        serde_json::from_value(parsed_val).map_err(|err| TransformError::SchemaMismatch {
            message: err.to_string(),
        })?;

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    mod transform_evaluation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn evaluates_valid_jq_filter_into_findings() {
            let dir = tempdir().unwrap();
            let filter_file = dir.path().join("test.jq");
            fs::write(
                &filter_file,
                r#"
                map({
                    smell: "oversized-file",
                    details: {},
                    issues: [{
                        key: .path,
                        details: {
                            file: .path,
                            line: .lines
                        }
                    }]
                })
                "#,
            )
            .unwrap();

            let input = r#"[{"path": "src/lib.rs", "lines": 500}]"#;
            let findings = evaluate_jq(&filter_file, input).unwrap();

            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0].smell, "oversized-file");
            assert_eq!(findings[0].issues[0].key, "src/lib.rs");
            assert_eq!(findings[0].issues[0].details.file, "src/lib.rs");
            assert_eq!(findings[0].issues[0].details.line, Some(500));
        }

        #[test]
        fn rejects_transform_producing_non_array() {
            let dir = tempdir().unwrap();
            let filter_file = dir.path().join("not_array.jq");
            fs::write(&filter_file, r#"{"error": "not an array"}"#).unwrap();

            let input = r#"{"test": 1}"#;
            let err = evaluate_jq(&filter_file, input).unwrap_err();
            match err {
                TransformError::NotAnArray { .. } => {}
                other => panic!("expected NotAnArray error, got {:?}", other),
            }
        }
    }
}
