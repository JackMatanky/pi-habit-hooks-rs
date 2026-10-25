import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

function findPackageRoot(startDir: string): string {
  let current = startDir;
  while (current !== dirname(current)) {
    if (existsSync(join(current, "package.json")) || existsSync(join(current, "Cargo.toml"))) {
      return current;
    }
    current = dirname(current);
  }
  return startDir;
}

const PACKAGE_ROOT = findPackageRoot(__dirname);

export function resolveBinary(name: "habit-hooks" | "habit-snooze" | "habit-sensors" | "habit-mapper"): string {
  const candidates = [
    join(PACKAGE_ROOT, "target", "release", name),
    join(PACKAGE_ROOT, "target", "debug", name),
  ];

  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  // Fallback to name in PATH
  return name;
}

export interface RunHabitHooksOptions {
  scope?: "working" | "staged" | "all" | "since";
  sinceRef?: string;
  files?: string[];
  fix?: boolean;
  cwd?: string;
}

export interface HabitHooksResult {
  exitCode: number;
  output: string;
  passed: boolean;
}

export function runHabitHooks(options: RunHabitHooksOptions = {}): HabitHooksResult {
  const cwd = options.cwd ?? process.cwd();

  // If auto-fix requested, run cargo clippy --fix first
  if (options.fix) {
    spawnSync("cargo", ["clippy", "--fix", "--allow-dirty", "--allow-no-vcs"], {
      cwd,
      stdio: "ignore",
    });
  }

  const binary = resolveBinary("habit-hooks");
  const args: string[] = [];

  switch (options.scope) {
    case "all":
      args.push("--all");
      break;
    case "since":
      if (options.sinceRef) {
        args.push("--since", options.sinceRef);
      }
      break;
    case "working":
    case "staged":
    default:
      args.push("--changed");
      break;
  }

  if (options.files && options.files.length > 0) {
    for (const f of options.files) {
      args.push("--file", f);
    }
  }

  const proc = spawnSync(binary, args, {
    cwd,
    encoding: "utf-8",
    env: {
      ...process.env,
      HABIT_HOOKS_PLUGINS_DIR: join(PACKAGE_ROOT, "plugins"),
    },
  });

  const output = (proc.stdout || "") + (proc.stderr || "");
  const exitCode = proc.status ?? (proc.error ? 1 : 0);

  return {
    exitCode,
    output,
    passed: exitCode === 0,
  };
}

export interface RunHabitSnoozeOptions {
  action: "list" | "add" | "prune";
  reason?: string;
  cwd?: string;
  inputJson?: string;
}

export function runHabitSnooze(options: RunHabitSnoozeOptions): { exitCode: number; output: string } {
  const cwd = options.cwd ?? process.cwd();
  const binary = resolveBinary("habit-snooze");
  const args: string[] = [];

  if (options.action === "list") {
    args.push("--list");
  } else if (options.action === "add") {
    args.push("--snooze");
    if (options.reason) {
      args.push("--reason", options.reason);
    }
  } else if (options.action === "prune") {
    args.push("--prune");
  }

  const proc = spawnSync(binary, args, {
    cwd,
    input: options.inputJson ?? "[]",
    encoding: "utf-8",
    env: {
      ...process.env,
      HABIT_HOOKS_PLUGINS_DIR: join(PACKAGE_ROOT, "plugins"),
    },
  });

  const output = (proc.stdout || "") + (proc.stderr || "");
  const exitCode = proc.status ?? 0;

  return { exitCode, output };
}
