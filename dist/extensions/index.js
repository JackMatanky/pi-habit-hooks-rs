import { spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { runHabitHooks, runHabitSnooze } from "./runner.js";
function hasRustFiles(root) {
    if (!fs.existsSync(path.join(root, "Cargo.toml")))
        return false;
    const dirs = [root];
    while (dirs.length) {
        const dir = dirs.pop();
        let entries;
        try {
            entries = fs.readdirSync(dir, { withFileTypes: true });
        }
        catch {
            continue;
        }
        for (const entry of entries) {
            if (entry.isFile() && entry.name.endsWith(".rs"))
                return true;
            if (entry.isDirectory() && ![".git", ".habit-hooks", "target", "node_modules"].includes(entry.name)) {
                dirs.push(path.join(dir, entry.name));
            }
        }
    }
    return false;
}
function hasChangedRustFiles(cwd) {
    const status = spawnSync("git", ["status", "--porcelain", "-z", "--untracked-files=all", "--", "*.rs"], {
        cwd,
        encoding: "utf-8",
    });
    return status.status !== 0 || status.stdout.length > 0;
}
const SYSTEM_GUIDELINES = `
## Habit Hooks Quality Coaching Rules
1. Before completing a turn that modifies Rust source files, run 'habit_hooks' to ensure no code smells were introduced.
2. If violations are reported, refactor the code according to the outcome-oriented guidance before yielding.
3. If suppression is strictly necessary (e.g. FFI boundary), always use scoped '#[expect(lint, reason = "...")]' with an explicit technical justification. NEVER use '#![allow(...)]' at the crate or module level.
4. If a smell cannot be immediately refactored, use 'habit_snooze' with an explicit reason. Note that snoozes lapse automatically whenever the affected file is modified.
`;
export async function validateUnwrapWithLaya(finding, pi) {
    if (finding.smell !== "unhandled-unwrap" || typeof pi.callTool !== "function") {
        return undefined;
    }
    const content = finding.issues?.[0]?.details?.content ?? "";
    try {
        const resp = (await pi.callTool("mcp__laya_predict", {
            state: { text: content },
            questions: {
                is_safe: {
                    type: "choice",
                    instructions: "Is this unwrap safe due to a local invariant or guard?",
                    criteria: { safe: "The unwrap is safely guarded", unsafe: "The unwrap may panic" },
                },
            },
        }));
        if (resp?.answers?.is_safe?.choice === "safe" && (resp.answers.is_safe.confidence ?? 0) >= 0.6) {
            return true;
        }
    }
    catch {
        // Silently continue if Laya MCP is not mounted
    }
    return false;
}
export default function (pi) {
    // 1. Register habit_hooks tool
    pi.registerTool({
        name: "habit_hooks",
        label: "Habit Hooks",
        description: "Run Habit Hooks Rust code-quality coaching over current working changes, staged changes, or the entire codebase.",
        parameters: {
            type: "object",
            properties: {
                scope: {
                    type: "string",
                    enum: ["working", "staged", "all", "since"],
                    description: "Scope of files to scan. Defaults to 'working'.",
                    default: "working",
                },
                sinceRef: {
                    type: "string",
                    description: "Git ref to compare against when scope is 'since' (e.g. 'main' or 'HEAD~1').",
                },
                files: {
                    type: "array",
                    items: { type: "string" },
                    description: "Specific file paths to scan.",
                },
                fix: {
                    type: "boolean",
                    description: "Attempt safe auto-fix (e.g. cargo clippy --fix) before running checks.",
                    default: false,
                },
            },
        },
        // Standard Pi/OMP execute method
        async execute(_toolCallId, params, _signal, _onUpdate, ctx) {
            const cwd = ctx?.cwd ?? process.cwd();
            const scope = typeof params.scope === "string" ? params.scope : "working";
            const sinceRef = typeof params.sinceRef === "string" ? params.sinceRef : undefined;
            const files = Array.isArray(params.files) ? params.files : undefined;
            const fix = typeof params.fix === "boolean" ? params.fix : false;
            const result = runHabitHooks({
                scope,
                sinceRef,
                files,
                fix,
                cwd,
            });
            if (ctx?.ui?.setStatus) {
                const statusText = result.passed ? "✅ clean" : "⚠️ coaching active";
                ctx.ui.setStatus("habit-hooks", statusText);
            }
            return {
                content: [{ type: "text", text: result.output }],
                details: { exitCode: result.exitCode, passed: result.passed },
                exitCode: result.exitCode,
                passed: result.passed,
                guidance: result.output,
            };
        },
        // Backward-compatible handler method
        async handler(args, context) {
            const cwd = context?.cwd ?? process.cwd();
            const scope = typeof args.scope === "string" ? args.scope : "working";
            const sinceRef = typeof args.sinceRef === "string" ? args.sinceRef : undefined;
            const files = Array.isArray(args.files) ? args.files : undefined;
            const fix = typeof args.fix === "boolean" ? args.fix : false;
            const result = runHabitHooks({
                scope,
                sinceRef,
                files,
                fix,
                cwd,
            });
            if (context?.ui?.setStatus) {
                const statusText = result.passed ? "✅ clean" : "⚠️ coaching active";
                context.ui.setStatus("habit-hooks", statusText);
            }
            return {
                content: [{ type: "text", text: result.output }],
                details: { exitCode: result.exitCode, passed: result.passed },
                exitCode: result.exitCode,
                passed: result.passed,
                guidance: result.output,
            };
        },
    });
    // 2. Register habit_snooze tool
    pi.registerTool({
        name: "habit_snooze",
        label: "Habit Snooze",
        description: "Manage Habit Hooks smell exemptions and snooze entries. Snoozes automatically lapse when the file's Git blob SHA changes.",
        parameters: {
            type: "object",
            properties: {
                action: {
                    type: "string",
                    enum: ["list", "add", "prune"],
                    description: "Action to perform: 'list' active snoozes, 'add' findings to snooze, or 'prune' dead snoozes.",
                    default: "list",
                },
                reason: {
                    type: "string",
                    description: "Documented technical justification when adding a snooze exemption.",
                },
                inputJson: {
                    type: "string",
                    description: "JSON array of findings to snooze when action is 'add'.",
                },
            },
            required: ["action"],
        },
        async execute(_toolCallId, params, _signal, _onUpdate, ctx) {
            const cwd = ctx?.cwd ?? process.cwd();
            const action = typeof params.action === "string" ? params.action : "list";
            const reason = typeof params.reason === "string" ? params.reason : undefined;
            const inputJson = typeof params.inputJson === "string" ? params.inputJson : undefined;
            const result = runHabitSnooze({
                action,
                reason,
                inputJson,
                cwd,
            });
            return {
                content: [{ type: "text", text: result.output }],
                details: { exitCode: result.exitCode },
                exitCode: result.exitCode,
                output: result.output,
            };
        },
        async handler(args, context) {
            const cwd = context?.cwd ?? process.cwd();
            const action = typeof args.action === "string" ? args.action : "list";
            const reason = typeof args.reason === "string" ? args.reason : undefined;
            const inputJson = typeof args.inputJson === "string" ? args.inputJson : undefined;
            const result = runHabitSnooze({
                action,
                reason,
                inputJson,
                cwd,
            });
            return {
                content: [{ type: "text", text: result.output }],
                details: { exitCode: result.exitCode },
                exitCode: result.exitCode,
                output: result.output,
            };
        },
    });
    // 3. Register Slash Commands (supports both Pi string-name signature and object signature)
    if (typeof pi.registerCommand === "function") {
        const registerCmd = (name, description, handler) => {
            try {
                if (typeof pi.registerCommand === "function") {
                    if (pi.registerCommand.length === 1) {
                        pi.registerCommand({ name, description, handler });
                    }
                    else if (pi.registerCommand.length === 3) {
                        pi.registerCommand(name, description, handler);
                    }
                    else {
                        pi.registerCommand(name, { description, handler });
                    }
                }
            }
            catch {
                try {
                    pi.registerCommand({ name, description, handler });
                }
                catch {
                    // Ignore if host doesn't support command registration
                }
            }
        };
        registerCmd("habit-hooks", "Run Habit Hooks coaching checks on working tree", async (args, context) => {
            const cwd = context?.cwd ?? process.cwd();
            const scope = args.includes("--all") ? "all" : "working";
            const result = runHabitHooks({ scope, cwd });
            context?.ui?.log?.(result.output);
        });
        registerCmd("snooze", "Manage Habit Hooks snoozed exemptions", async (args, context) => {
            const cwd = context?.cwd ?? process.cwd();
            const action = args.includes("--prune") ? "prune" : "list";
            const result = runHabitSnooze({ action, cwd });
            context?.ui?.log?.(result.output);
        });
    }
    // Only introduce automatic coaching instructions in an active Rust project.
    if (typeof pi.on === "function") {
        pi.on("before_agent_start", (event, ctx) => {
            const cwd = ctx?.cwd ?? event?.cwd ?? process.cwd();
            if (!hasRustFiles(cwd))
                return;
            const prompt = event.systemPrompt;
            if (Array.isArray(prompt)) {
                if (!prompt.some((b) => b.includes("Habit Hooks Quality Coaching Rules"))) {
                    return { systemPrompt: [...prompt, SYSTEM_GUIDELINES] };
                }
            }
            else if (typeof prompt === "string") {
                if (!prompt.includes("Habit Hooks Quality Coaching Rules")) {
                    return { systemPrompt: `${prompt}\n\n${SYSTEM_GUIDELINES}` };
                }
            }
        });
    }
    // 5. Event-Driven Lifecycle: tool_call incremental scan & status badge update
    if (typeof pi.on === "function") {
        pi.on("tool_call", async (data, ctxArg) => {
            const toolName = data?.tool ?? data?.name ?? data?.toolName;
            const params = data?.params ?? data?.input ?? data?.args;
            const filePath = params?.path ?? params?.filePath ?? params?.file;
            if (!filePath || typeof filePath !== "string") {
                return;
            }
            const isEditOrWrite = toolName === "edit" || toolName === "write" || !toolName;
            const isTargetLang = /\.rs$/i.test(filePath);
            if (isEditOrWrite && isTargetLang) {
                const ctx = ctxArg ?? data?.ctx;
                const cwd = ctx?.cwd ?? data?.cwd ?? process.cwd();
                if (!hasRustFiles(cwd))
                    return;
                const result = runHabitHooks({ scope: "working", files: [filePath], cwd });
                const smellHeaders = (result.output.match(/^###\s+[a-z0-9-]+/gm) || []).length;
                const count = smellHeaders > 0 ? smellHeaders : (result.passed ? 0 : 1);
                const setStatus = ctx?.ui?.setStatus ?? pi.ui?.setStatus;
                if (typeof setStatus === "function") {
                    const statusText = result.passed && count === 0 ? "✅ clean" : `⚠️ ${count} smell(s)`;
                    setStatus("habit-hooks", statusText);
                }
                if (!result.passed && count > 0) {
                    return {
                        additionalContext: `⚠️ Habit Hooks coaching feedback for ${filePath}:\n\n${result.output}`,
                    };
                }
            }
        });
        // 6. Pre-Yield Quality Gate: turn_end / before_yield / session_stop intercept
        const preYieldGateHandler = (data, ctxArg) => {
            const ctx = ctxArg ?? data?.ctx;
            const cwd = ctx?.cwd ?? data?.cwd ?? process.cwd();
            if (!hasRustFiles(cwd) || !hasChangedRustFiles(cwd)) {
                return { interrupt: false };
            }
            const result = runHabitHooks({ scope: "working", cwd });
            if (result.exitCode === 1) {
                const feedback = `⚠️ Habit Hooks Quality Gate: Enforced code smell detected...\n\n${result.output}\n\nPlease refactor before completing.`;
                return {
                    interrupt: true,
                    decision: "block",
                    reason: feedback,
                    message: feedback,
                };
            }
            return { interrupt: false };
        };
        pi.on("turn_end", preYieldGateHandler);
        pi.on("before_yield", preYieldGateHandler);
        pi.on("session_stop", preYieldGateHandler);
    }
}
