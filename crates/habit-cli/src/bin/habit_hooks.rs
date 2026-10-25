use clap::{Parser, Subcommand};
use habit_cli::{find_project_dir, load_enabled_plugins, load_project_config};
use habit_mapper::evaluate_and_render;
use habit_sensors::{detector::DetectorResolver, runner::SensorRunner, scope::ScopeArgs};
use habit_snooze::{filter_snoozed_findings, load_snooze_file};
use std::fs;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "habit-hooks", about = "Habit Hooks Code Quality Coaching Engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, help = "Scan all project files")]
    all: bool,

    #[arg(long = "file", help = "Scan specific files")]
    files: Vec<String>,

    #[arg(long, help = "Scan files changed relative to branch base")]
    branch: Option<Option<String>>,

    #[arg(long, help = "Scan files changed since git ref")]
    since: Option<String>,

    #[arg(long, help = "Scan files changed in last N commits")]
    last: Option<usize>,

    #[arg(long, help = "Scan working tree modified files")]
    changed: bool,

    #[arg(long = "no-snooze", help = "Do not filter out snoozed issues")]
    no_snooze: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Initialize Habit Hooks configuration and verify tools")]
    Init,
}

fn main() {
    let cli = Cli::parse();
    let project_dir = find_project_dir();

    if let Some(Commands::Init) = cli.command {
        run_init(&project_dir);
        return;
    }

    let project_config = load_project_config(&project_dir);
    let plugins = load_enabled_plugins(&project_dir, &project_config);

    let scope_args = ScopeArgs {
        all: cli.all,
        files: cli.files,
        branch: cli.branch,
        since: cli.since,
        last: cli.last,
        changed: cli.changed,
        no_snooze: cli.no_snooze,
    };

    let runner = SensorRunner::new(&project_dir, &project_config, &plugins, &scope_args);
    let findings = runner.run_all();

    let final_findings = if cli.no_snooze {
        findings
    } else {
        match load_snooze_file(&project_dir) {
            Ok(snooze) => filter_snoozed_findings(findings, &snooze, &project_dir),
            Err(_) => findings,
        }
    };

    let findings_json = serde_json::to_string(&final_findings).unwrap_or_else(|err| {
        eprintln!("habit-hooks: failed serializing findings: {err}");
        process::exit(1);
    });

    let plugin_dirs: Vec<_> = plugins.iter().map(|p| p.plugin_dir.clone()).collect();
    let result = evaluate_and_render(
        &findings_json,
        &project_dir,
        &plugin_dirs,
        &project_config,
    );

    print!("{}", result.output);
    process::exit(result.exit_code);
}

fn run_init(project_dir: &std::path::Path) {
    println!("Initializing Habit Hooks in {}...", project_dir.display());

    let habit_dir = project_dir.join(".habit-hooks");
    if let Err(err) = fs::create_dir_all(&habit_dir) {
        eprintln!("Failed to create .habit-hooks directory: {err}");
        process::exit(1);
    }

    let config_file = habit_dir.join("config.toml");
    let has_rust = project_dir.join("Cargo.toml").is_file()
        || walkdir::WalkDir::new(project_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| e.path().extension().is_some_and(|ext| ext == "rs"));

    let plugins_list = if has_rust {
        r#"plugins = ["rust", "generic"]"#
    } else {
        r#"plugins = ["generic"]"#
    };

    if !config_file.is_file() {
        let default_config = format!(
            r#"# Habit Hooks Configuration
{plugins_list}
uncoached = "suggest"

[scope]
changedOnly = false
autoBranchOffMain = false
branchBase = "main"
mainBranch = "main"
"#
        );
        if let Err(err) = fs::write(&config_file, default_config) {
            eprintln!("Failed to write config file: {err}");
            process::exit(1);
        }
        println!("Created {}", config_file.display());
    } else {
        println!("Existing configuration found at {}", config_file.display());
    }

    // Verify detectors across enabled plugins
    let project_config = load_project_config(project_dir);
    let plugins = load_enabled_plugins(project_dir, &project_config);

    println!("\nVerifying tool detectors for enabled plugins:");
    let mut missing_detectors = Vec::new();

    for plugin in &plugins {
        let resolver = DetectorResolver::new(project_dir, &plugin.plugin_dir, &plugin.detectors);
        for detector in &plugin.detectors {
            match resolver.resolve_tool(&detector.name) {
                Ok(path) => {
                    println!("  [OK] {:<16} ({})", detector.name, path.display());
                }
                Err(_) => {
                    println!("  [MISSING] {:<16}", detector.name);
                    if let Some(ref install) = detector.install {
                        println!("            Install: {install}");
                        missing_detectors.push((detector.name.clone(), install.clone()));
                    }
                }
            }
        }
    }

    if missing_detectors.is_empty() {
        println!("\n✅ All declared detectors are installed and ready.");
    } else {
        println!("\n⚠️ Some detectors are not installed yet. You can run the install commands above.");
    }
}
