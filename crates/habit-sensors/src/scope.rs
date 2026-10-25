use glob::Pattern;
use habit_core::ScopeConfig;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Default)]
pub struct ScopeArgs {
    pub all: bool,
    pub files: Vec<String>,
    pub branch: Option<Option<String>>, // --branch [base]
    pub since: Option<String>,
    pub last: Option<usize>,
    pub changed: bool,
    pub no_snooze: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ScopeError {
    #[error("failed to run git command: {message}")]
    GitError { message: String },
}

pub fn matches_globs(relative_path: &str, globs: &[String]) -> bool {
    if globs.is_empty() {
        return true;
    }

    let mut is_matched = false;
    let normalized = relative_path.replace('\\', "/");

    for glob in globs {
        if let Some(negative) = glob.strip_prefix('!') {
            if let Ok(pattern) = Pattern::new(negative) {
                if pattern.matches(&normalized) {
                    return false;
                }
            }
        } else if let Ok(pattern) = Pattern::new(glob) {
            if pattern.matches(&normalized) {
                is_matched = true;
            }
        }
    }

    is_matched
}

pub fn filter_files_by_globs(files: &[String], globs: &[String]) -> Vec<String> {
    if globs.is_empty() {
        return files.to_vec();
    }
    files
        .iter()
        .filter(|f| matches_globs(f, globs))
        .cloned()
        .collect()
}

pub fn discover_files(
    project_dir: &Path,
    args: &ScopeArgs,
    config: &ScopeConfig,
    plugin_globs: &[String],
) -> Result<Vec<String>, ScopeError> {
    let canonical_root = project_dir
        .canonicalize()
        .unwrap_or_else(|_| project_dir.to_path_buf());

    // 1. If explicit files are given via --file
    if !args.files.is_empty() {
        let mut results = Vec::new();
        for f in &args.files {
            let path = PathBuf::from(f);
            let rel = if path.is_absolute() {
                habit_core::normalize_to_relative_slash(&canonical_root, f)
                    .unwrap_or_else(|_| f.clone())
            } else {
                f.replace('\\', "/")
            };
            if canonical_root.join(&rel).is_file() {
                results.push(rel);
            }
        }
        return Ok(filter_files_by_globs(&results, plugin_globs));
    }

    // 2. Check if git diff mode applies
    let is_git_mode = args.changed
        || args.branch.is_some()
        || args.since.is_some()
        || args.last.is_some()
        || (!args.all && config.changed_only);

    if is_git_mode && canonical_root.join(".git").exists() {
        let git_files = query_git_files(&canonical_root, args, config)?;
        let mut existing_files = BTreeSet::new();

        for file in git_files {
            let clean = file.trim().replace('\\', "/");
            if clean.is_empty() {
                continue;
            }
            if canonical_root.join(&clean).is_file() {
                existing_files.insert(clean);
            }
        }

        let file_list: Vec<String> = existing_files.into_iter().collect();
        return Ok(filter_files_by_globs(&file_list, plugin_globs));
    }

    // 3. Fallback to walking directory
    let mut files = Vec::new();
    for entry in WalkDir::new(&canonical_root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "target" && name != "node_modules" && name != ".habit-hooks"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(rel) = entry.path().strip_prefix(&canonical_root) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                files.push(rel_str);
            }
        }
    }

    Ok(filter_files_by_globs(&files, plugin_globs))
}

fn query_git_files(
    project_dir: &Path,
    args: &ScopeArgs,
    config: &ScopeConfig,
) -> Result<Vec<String>, ScopeError> {
    let mut files = BTreeSet::new();

    if let Some(ref since) = args.since {
        let diff = run_git(project_dir, &["diff", "--name-only", &format!("{since}...HEAD")])?;
        for line in diff.lines() {
            files.insert(line.trim().to_string());
        }
        return Ok(files.into_iter().collect());
    }

    if let Some(n) = args.last {
        let diff = run_git(
            project_dir,
            &["diff", "--name-only", &format!("HEAD~{n}...HEAD")],
        )?;
        for line in diff.lines() {
            files.insert(line.trim().to_string());
        }
        return Ok(files.into_iter().collect());
    }

    if let Some(ref branch_opt) = args.branch {
        let base = branch_opt.as_deref().unwrap_or(&config.branch_base);
        let diff = run_git(project_dir, &["diff", "--name-only", &format!("{base}...HEAD")])?;
        for line in diff.lines() {
            files.insert(line.trim().to_string());
        }
        return Ok(files.into_iter().collect());
    }

    // Default changed files: git status (working tree + index)
    let status = run_git(project_dir, &["status", "--porcelain"])?;
    for line in status.lines() {
        if line.len() > 3 {
            let path_part = &line[3..].trim();
            // Handle rename e.g. "R  old -> new"
            let target = if let Some((_old, new)) = path_part.split_once(" -> ") {
                new.trim()
            } else {
                path_part
            };
            files.insert(target.to_string());
        }
    }

    Ok(files.into_iter().collect())
}

fn run_git(project_dir: &Path, args: &[&str]) -> Result<String, ScopeError> {
    let output = Command::new("git")
        .current_dir(project_dir)
        .args(args)
        .output()
        .map_err(|err| ScopeError::GitError {
            message: err.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ScopeError::GitError {
            message: stderr.to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod glob_matching {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn matches_positive_and_respects_negative_globs() {
            let globs = vec![
                "**/*.rs".to_string(),
                "!**/target/**/*.rs".to_string(),
                "!**/tests/sample/**".to_string(),
            ];

            assert_eq!(matches_globs("src/main.rs", &globs), true);
            assert_eq!(matches_globs("src/lib.rs", &globs), true);
            assert_eq!(matches_globs("target/debug/build.rs", &globs), false);
            assert_eq!(matches_globs("tests/sample/complex.rs", &globs), false);
            assert_eq!(matches_globs("README.md", &globs), false);
        }

        #[test]
        fn filters_file_list_accurately() {
            let files = vec![
                "src/main.rs".to_string(),
                "src/lib.rs".to_string(),
                "target/foo.rs".to_string(),
                "package.json".to_string(),
            ];
            let globs = vec!["**/*.rs".to_string(), "!target/**".to_string()];

            let filtered = filter_files_by_globs(&files, &globs);
            assert_eq!(
                filtered,
                vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]
            );
        }
    }
}
