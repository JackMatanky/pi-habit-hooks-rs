import { runHabitHooks, runHabitSnooze } from "./runner.js";
export default function (pi) {
    // 1. Register habit_hooks tool
    pi.registerTool({
        name: "habit_hooks",
        description: "Run Habit Hooks code-quality coaching engine over current working changes, staged changes, or entire codebase. Translates linter and AST findings into canonical smell coaching prompts.",
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
        handler: async (args, context) => {
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
                if (result.passed) {
                    context.ui.setStatus("habit-hooks", { text: "✅ clean" });
                }
                else {
                    context.ui.setStatus("habit-hooks", { text: "⚠️ coaching active" });
                }
            }
            return {
                exitCode: result.exitCode,
                passed: result.passed,
                guidance: result.output,
            };
        },
    });
    // 2. Register habit_snooze tool
    pi.registerTool({
        name: "habit_snooze",
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
        handler: async (args, context) => {
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
                exitCode: result.exitCode,
                output: result.output,
            };
        },
    });
    // 3. Register Slash Commands
    if (pi.registerCommand) {
        pi.registerCommand({
            name: "habit-hooks",
            description: "Run Habit Hooks coaching checks on working tree",
            handler: async (args, context) => {
                const cwd = context?.cwd ?? process.cwd();
                const scope = args.includes("--all") ? "all" : "working";
                const result = runHabitHooks({ scope, cwd });
                context?.ui?.log?.(result.output);
            },
        });
        pi.registerCommand({
            name: "snooze",
            description: "Manage Habit Hooks snoozed exemptions",
            handler: async (args, context) => {
                const cwd = context?.cwd ?? process.cwd();
                const action = args.includes("--prune") ? "prune" : "list";
                const result = runHabitSnooze({ action, cwd });
                context?.ui?.log?.(result.output);
            },
        });
    }
    // 4. Inject System Prompt Guidelines
    if (pi.injectSystemPrompt) {
        pi.injectSystemPrompt(`
## Habit Hooks Quality Coaching Rules
1. Before completing any non-trivial code modification turn, run 'habit_hooks' to ensure no code smells were introduced.
2. If violations are reported, refactor the code according to the outcome-oriented guidance before yielding.
3. If suppression is strictly necessary (e.g. FFI boundary), always use scoped '#[expect(lint, reason = "...")]' with an explicit technical justification. NEVER use '#![allow(...)]' at the crate or module level.
4. If a smell cannot be immediately refactored, use 'habit_snooze' with an explicit reason. Note that snoozes lapse automatically whenever the affected file is modified.
`);
    }
}
