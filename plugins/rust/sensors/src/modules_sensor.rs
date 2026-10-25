use clap::Parser;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "modules_sensor", about = "Cargo Modules Sensor for Habit Hooks")]
struct Args {
    #[arg(num_args = 0..)]
    files: Vec<String>,
}

fn find_cargo_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
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

fn check_orphans_natively(root: &Path, filter_files: &BTreeSet<String>) -> Vec<serde_json::Value> {
    let mut violations = Vec::new();
    let src_dir = root.join("src");
    if !src_dir.is_dir() {
        return violations;
    }

    // Collect all declared modules in the crate
    let mut declared_mods = BTreeSet::new();
    let mut all_rs_files = Vec::new();

    for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "rs") {
            all_rs_files.push(entry.path().to_path_buf());
            if let Ok(content) = fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        for (idx, part) in parts.iter().enumerate() {
                            if *part == "mod" && idx + 1 < parts.len() {
                                let mod_name = parts[idx + 1].trim_end_matches(';').trim();
                                declared_mods.insert(mod_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    for file_path in all_rs_files {
        let file_name = file_path.file_stem().unwrap_or_default().to_string_lossy();
        if file_name == "main" || file_name == "lib" || file_name == "mod" {
            continue;
        }

        let rel_path = file_path
            .strip_prefix(root)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .replace('\\', "/");

        if !filter_files.is_empty() && !filter_files.contains(&rel_path) {
            continue;
        }

        if !declared_mods.contains(file_name.as_ref()) {
            violations.push(serde_json::json!({
                "kind": "orphan",
                "file": rel_path,
                "message": format!("Source file '{}' is not declared with 'mod {};' in any parent module", file_name, file_name)
            }));
        }
    }

    violations
}

fn main() {
    let args = Args::parse();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let root = find_cargo_root(&cwd).unwrap_or_else(|| cwd.clone());

    let files_set: BTreeSet<String> = args
        .files
        .iter()
        .map(|f| f.replace('\\', "/").trim_start_matches("./").to_string())
        .collect();

    let mut violations = Vec::new();

    // Check cargo-modules orphans if installed
    let modules_cmd = Command::new("cargo")
        .current_dir(&root)
        .args(["modules", "orphans", "--lib"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    if let Ok(output) = modules_cmd {
        if output.status.success() {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            for line in stdout_str.lines() {
                let trimmed = line.trim();
                if trimmed.ends_with(".rs") {
                    let rel = trimmed.replace('\\', "/");
                    if files_set.is_empty() || files_set.contains(&rel) {
                        violations.push(serde_json::json!({
                            "kind": "orphan",
                            "file": rel,
                            "message": format!("Orphan module file '{rel}' is not connected to crate root")
                        }));
                    }
                }
            }
        } else {
            // Run native orphan check fallback
            violations.extend(check_orphans_natively(&root, &files_set));
        }
    } else {
        // Fallback to native orphan check
        violations.extend(check_orphans_natively(&root, &files_set));
    }

    let json = serde_json::to_string(&violations).unwrap_or_else(|_| "[]".to_string());
    println!("{json}");
}
