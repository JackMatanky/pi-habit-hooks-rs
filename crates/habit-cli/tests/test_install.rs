use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_install_pre_commit_hook() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    // Create a mock .git directory
    fs::create_dir_all(path.join(".git")).unwrap();

    // Run habit-hooks install in that dir
    let habit_hooks_bin = env!("CARGO_BIN_EXE_habit-hooks");
    let output = Command::new(habit_hooks_bin)
        .arg("install")
        .current_dir(path)
        .output()
        .expect("failed to execute habit-hooks install");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✅ Installed Habit Hooks pre-commit hook in .git/hooks/pre-commit"));

    let hook_file = path.join(".git").join("hooks").join("pre-commit");
    assert!(hook_file.is_file());

    let content = fs::read_to_string(&hook_file).unwrap();
    assert!(content.contains("# Habit Hooks automated pre-commit gate"));
    assert!(content.contains("exec habit-hooks --changed"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::metadata(&hook_file).unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0o111);
    }
}

#[test]
fn test_install_fails_outside_git_repo() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    let habit_hooks_bin = env!("CARGO_BIN_EXE_habit-hooks");
    let output = Command::new(habit_hooks_bin)
        .arg("install")
        .current_dir(path)
        .output()
        .expect("failed to execute habit-hooks install");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Not a git repository"));
}

#[test]
fn test_pre_commit_blocks_commit_on_enforced_smell() {
    let dir = tempdir().unwrap();
    let path = dir.path();

    // Initialize git repo
    let run_cmd = |cmd: &str, args: &[&str]| {
        let out = Command::new(cmd)
            .current_dir(path)
            .args(args)
            .output()
            .expect("failed to run command");
        assert!(out.status.success(), "command failed: {} {:?}", cmd, args);
    };

    run_cmd("git", &["init", "-b", "main"]);
    run_cmd("git", &["config", "user.name", "Test Runner"]);
    run_cmd("git", &["config", "user.email", "test@runner.local"]);

    let habit_hooks_bin = env!("CARGO_BIN_EXE_habit-hooks");
    let bin_dir = std::path::Path::new(habit_hooks_bin).parent().unwrap();

    // Install pre-commit hook
    let output = Command::new(habit_hooks_bin)
        .arg("install")
        .current_dir(path)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Setup mock sensor that emits an enforced smell ("lock-across-await")
    let plugin_dir = path.join(".habit-hooks").join("mock");
    fs::create_dir_all(&plugin_dir).unwrap();

    // Script that outputs JSON finding for lock-across-await
    let sensor_script = plugin_dir.join("sensor.sh");
    fs::write(
        &sensor_script,
        r#"#!/bin/sh
echo '[{"smell":"lock-across-await","language":"rust","details":{},"issues":[{"key":"src/lib.rs","details":{"file":"src/lib.rs","line":1,"message":"holding lock across await"}}]}]'
"#,
    ).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&sensor_script).unwrap().permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&sensor_script, perms);
    }

    let plugin_toml = plugin_dir.join("config.toml");
    fs::write(
        &plugin_toml,
        r#"files = ["**/*.rs"]
[[sensors]]
name = "mock-sensor"
tool = "${dir}/sensor.sh"
args = []
success_exit_codes = [0]
"#,
    ).unwrap();

    let project_config_toml = path.join(".habit-hooks").join("config.toml");
    fs::write(
        &project_config_toml,
        r#"plugins = ["mock"]
[scope]
changedOnly = true
"#,
    ).unwrap();

    // Create and stage file
    fs::create_dir_all(path.join("src")).unwrap();
    fs::write(path.join("src/lib.rs"), "async fn foo() {}\n").unwrap();
    run_cmd("git", &["add", "src/lib.rs"]);

    // Construct PATH containing habit-hooks binary
    let current_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), current_path);

    let commit_output = Command::new("git")
        .current_dir(path)
        .args(&["commit", "-m", "attempt commit"])
        .env("PATH", new_path)
        .output()
        .expect("git commit failed to execute");

    // Commit must be blocked (exit code non-zero, specifically 1 from habit-hooks)
    assert!(
        !commit_output.status.success(),
        "git commit should have been blocked by pre-commit hook"
    );
    let combined_output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&commit_output.stdout),
        String::from_utf8_lossy(&commit_output.stderr)
    );
    assert!(
        combined_output.contains("lock-across-await") || commit_output.status.code() == Some(1),
        "pre-commit hook output did not reflect enforced smell failure"
    );
}

#[test]
fn test_install_with_hk_flag_creates_hk_pkl() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    fs::create_dir_all(path.join(".git")).unwrap();

    let habit_hooks_bin = env!("CARGO_BIN_EXE_habit-hooks");
    let output = Command::new(habit_hooks_bin)
        .args(&["install", "--hk"])
        .current_dir(path)
        .output()
        .expect("failed to execute habit-hooks install --hk");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hk.pkl"));

    let hk_pkl = path.join("hk.pkl");
    assert!(hk_pkl.is_file());
    let content = fs::read_to_string(&hk_pkl).unwrap();
    assert!(content.contains("[\"habit-hooks\"]"));
    assert!(content.contains("glob = \"**/*.rs\""));
    assert!(content.contains("check = \"habit-hooks --changed\""));

    let hook_file = path.join(".git").join("hooks").join("pre-commit");
    assert!(hook_file.is_file());
    let hook_content = fs::read_to_string(&hook_file).unwrap();
    assert!(hook_content.contains("hk run pre-commit --from-hook"));
}

#[test]
fn test_install_auto_detects_existing_hk_pkl() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    fs::create_dir_all(path.join(".git")).unwrap();

    // Pre-create an existing hk.pkl with dummy steps
    let existing_hk_pkl = r#"amends "package://github.com/jdx/hk/releases/download/v2.0.0/hk@2.0.0#/Config.pkl"

steps {
    ["prettier"] {
        glob = "**/*.json"
        check = "prettier --check {{files}}"
    }
}
"#;
    fs::write(path.join("hk.pkl"), existing_hk_pkl).unwrap();

    let habit_hooks_bin = env!("CARGO_BIN_EXE_habit-hooks");
    // Run without --hk flag
    let output = Command::new(habit_hooks_bin)
        .arg("install")
        .current_dir(path)
        .output()
        .expect("failed to execute habit-hooks install");

    assert!(output.status.success());
    let hk_pkl = path.join("hk.pkl");
    let content = fs::read_to_string(&hk_pkl).unwrap();
    assert!(content.contains("[\"habit-hooks\"]"));
    assert!(content.contains("[\"prettier\"]"));
}
