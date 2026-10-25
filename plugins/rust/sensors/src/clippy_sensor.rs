use clap::Parser;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(name = "clippy_sensor", about = "Cargo Clippy Sensor for Habit Hooks")]
struct Args {
    #[arg(long = "target-dir", default_value = ".habit-hooks/target")]
    target_dir: String,

    #[arg(long = "files", num_args = 0..)]
    files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DiagnosticSpan {
    file_name: String,
    line_start: usize,
    column_start: usize,
    is_primary: bool,
    text: Vec<SpanText>,
}

#[derive(Debug, Deserialize)]
struct SpanText {
    text: String,
}

#[derive(Debug, Deserialize)]
struct DiagnosticCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct DiagnosticMessage {
    code: Option<DiagnosticCode>,
    message: String,
    spans: Vec<DiagnosticSpan>,
    level: String,
}

#[derive(Debug, Deserialize)]
struct CompilerArtifact {
    reason: String,
    message: Option<DiagnosticMessage>,
}

fn find_cargo_root(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent().unwrap_or(start).to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        if current.join("Cargo.toml").is_file() {
            return Some(current);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    None
}

fn main() {
    let args = Args::parse();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let root = if let Some(first_file) = args.files.first() {
        let file_path = if Path::new(first_file).is_absolute() {
            PathBuf::from(first_file)
        } else {
            cwd.join(first_file)
        };
        find_cargo_root(&file_path).unwrap_or_else(|| cwd.clone())
    } else {
        find_cargo_root(&cwd).unwrap_or_else(|| cwd.clone())
    };

    let files_set: BTreeSet<String> = args
        .files
        .iter()
        .map(|f| f.replace('\\', "/").trim_start_matches("./").to_string())
        .collect();

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&root);
    cmd.args(["clippy", "--message-format=json", "--all-targets"]);
    cmd.env("CARGO_TARGET_DIR", &args.target_dir);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let output = match cmd.output() {
        Ok(o) => o,
        Err(err) => {
            eprintln!("clippy_sensor: failed to spawn cargo clippy: {err}");
            println!("[]");
            return;
        }
    };

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut normalized_violations = Vec::new();
    let mut seen_keys = BTreeSet::new();

    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let artifact: CompilerArtifact = match serde_json::from_str(trimmed) {
            Ok(a) => a,
            Err(_) => continue,
        };

        if artifact.reason != "compiler-message" {
            continue;
        }

        if let Some(diag) = artifact.message {
            if diag.level == "note" || diag.level == "help" {
                continue;
            }

            let code = diag.code.map(|c| c.code).unwrap_or_default();
            if code.is_empty() {
                continue;
            }

            for span in diag.spans {
                if !span.is_primary {
                    continue;
                }

                let span_path = Path::new(&span.file_name);
                let abs_span_file = if span_path.is_absolute() {
                    span_path.to_path_buf()
                } else {
                    root.join(span_path)
                };

                let clean_file = pathdiff::diff_paths(&abs_span_file, &cwd)
                    .unwrap_or_else(|| span_path.to_path_buf())
                    .to_string_lossy()
                    .replace('\\', "/")
                    .trim_start_matches("./")
                    .to_string();

                if !files_set.is_empty()
                    && !files_set.contains(&clean_file)
                    && !files_set.iter().any(|f| clean_file.ends_with(f) || f.ends_with(&clean_file))
                {
                    continue;
                }

                let dedupe_key = (code.clone(), clean_file.clone(), span.line_start, span.column_start);
                if !seen_keys.insert(dedupe_key) {
                    continue;
                }

                let content = span.text.first().map(|t| t.text.trim().to_string());

                normalized_violations.push(serde_json::json!({
                    "code": code,
                    "filename": clean_file,
                    "line": span.line_start,
                    "column": span.column_start,
                    "content": content,
                    "message": diag.message
                }));
            }
        }
    }

    let result_json = serde_json::to_string(&normalized_violations).unwrap_or_else(|_| "[]".to_string());
    println!("{result_json}");
}
