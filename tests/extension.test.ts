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
  const registeredTools: string[] = [];
  const registeredCommands: string[] = [];
  let promptInjected = false;

  const mockPi = {
    registerTool: (tool: { name: string }) => {
      registeredTools.push(tool.name);
    },
    registerCommand: (cmd: { name: string }) => {
      registeredCommands.push(cmd.name);
    },
    injectSystemPrompt: (_prompt: string) => {
      promptInjected = true;
    },
  };

  registerExtension(mockPi);

  assert.deepEqual(registeredTools, ["habit_hooks", "habit_snooze"]);
  assert.deepEqual(registeredCommands, ["habit-hooks", "snooze"]);
  assert.equal(promptInjected, true);
});
