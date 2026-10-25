import test from "node:test";
import assert from "node:assert/strict";
import { resolveBinary, runHabitHooks, runHabitSnooze } from "../extensions/runner.js";
import registerExtension from "../extensions/index.js";
test("resolveBinary resolves built CLI binaries", () => {
    const binary = resolveBinary("habit-hooks");
    assert.ok(binary.includes("habit-hooks"));
});
test("runHabitHooks runs on workspace and returns clean status or coaching", () => {
    const result = runHabitHooks({ scope: "working" });
    assert.equal(typeof result.exitCode, "number");
    assert.equal(typeof result.passed, "boolean");
    assert.ok(result.output.length > 0);
});
test("runHabitSnooze supports list action", () => {
    const result = runHabitSnooze({ action: "list" });
    assert.equal(typeof result.exitCode, "number");
    assert.ok(result.output.length >= 0);
});
test("registers tools and commands with host", () => {
    const registeredTools = [];
    const registeredCommands = [];
    let promptInjected = false;
    const mockPi = {
        registerTool: (tool) => {
            registeredTools.push(tool);
        },
        registerCommand: (...args) => {
            if (typeof args[0] === "string") {
                registeredCommands.push(args[0]);
            }
            else if (args[0] && typeof args[0] === "object" && "name" in args[0]) {
                registeredCommands.push(String(args[0].name));
            }
        },
        injectSystemPrompt: (_prompt) => {
            promptInjected = true;
        },
    };
    registerExtension(mockPi);
    const toolNames = registeredTools.map((t) => t.name);
    assert.deepEqual(toolNames, ["habit_hooks", "habit_snooze"]);
    assert.deepEqual(registeredCommands, ["habit-hooks", "snooze"]);
    assert.equal(promptInjected, true);
    // Verify both execute and handler exist on registered tools
    for (const t of registeredTools) {
        assert.equal(typeof t.execute, "function");
        assert.equal(typeof t.handler, "function");
    }
});
