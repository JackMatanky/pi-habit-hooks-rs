import test from "node:test";
import assert from "node:assert/strict";
import { resolveBinary, runHabitHooks, runHabitSnooze } from "../extensions/runner.js";
import registerExtension, { validateUnwrapWithLaya } from "../extensions/index.js";

test("resolveBinary resolves built CLI binaries", () => {
  const binaryHooks = resolveBinary("habit-hooks");
  assert.ok(binaryHooks.includes("habit-hooks"));
  const binaryJudge = resolveBinary("habit-judge");
  assert.ok(binaryJudge.includes("habit-judge"));
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
  const registeredTools: Record<string, unknown>[] = [];
  const registeredCommands: string[] = [];
  let promptInjected = false;

  const mockPi = {
    registerTool: (tool: Record<string, unknown>) => {
      registeredTools.push(tool);
    },
    registerCommand: (...args: unknown[]) => {
      if (typeof args[0] === "string") {
        registeredCommands.push(args[0]);
      } else if (args[0] && typeof args[0] === "object" && "name" in args[0]) {
        registeredCommands.push(String((args[0] as { name: unknown }).name));
      }
    },
    injectSystemPrompt: (_prompt: string) => {
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

test("lifecycle: tool_call event updates status badge", async () => {
  const handlers: Record<string, Array<Function>> = {};
  const statusUpdates: Array<{ key: string; status: string | { text: string } | undefined }> = [];

  const mockPi = {
    registerTool: () => {},
    ui: {
      setStatus: (key: string, status: string | { text: string } | undefined) => {
        statusUpdates.push({ key, status });
      },
    },
    on: (event: string, handler: Function) => {
      handlers[event] = handlers[event] || [];
      handlers[event].push(handler);
    },
  };

  registerExtension(mockPi);
  assert.ok(handlers["tool_call"] && handlers["tool_call"].length > 0);

  // Simulate tool_call on an existing rust file
  const toolCallHandler = handlers["tool_call"][0];
  await toolCallHandler({
    tool: "edit",
    params: { path: "crates/habit-core/src/lib.rs" },
  });

  assert.ok(statusUpdates.length > 0);
  assert.equal(statusUpdates[0].key, "habit-hooks");
  const statusVal = typeof statusUpdates[0].status === "object" ? statusUpdates[0].status?.text : statusUpdates[0].status;
  assert.ok(
    statusVal?.includes("clean") || statusVal?.includes("smell")
  );
});

test("lifecycle: turn_end pre-yield gate passes cleanly or interrupts on smell", async () => {
  const handlers: Record<string, Array<Function>> = {};

  const mockPi = {
    registerTool: () => {},
    on: (event: string, handler: Function) => {
      handlers[event] = handlers[event] || [];
      handlers[event].push(handler);
    },
  };

  registerExtension(mockPi);
  assert.ok(handlers["turn_end"] && handlers["turn_end"].length > 0);

  const turnEndHandler = handlers["turn_end"][0];
  const res = await turnEndHandler({});
  assert.equal(typeof res.interrupt, "boolean");
  if (res.interrupt) {
    assert.ok(res.message.includes("Habit Hooks Quality Gate"));
  }
});

test("lifecycle: validateUnwrapWithLaya invokes Laya judge if available", async () => {
  let calledTool = "";
  const mockPi = {
    registerTool: () => {},
    callTool: async (toolName: string, _args: Record<string, unknown>) => {
      calledTool = toolName;
      return {
        answers: {
          is_safe: {
            choice: "safe",
            confidence: 0.85,
          },
        },
      };
    },
  };

  const finding = {
    smell: "unhandled-unwrap",
    issues: [{ details: { content: "if !v.is_empty() { v.unwrap(); }" } }],
  };

  const isSafe = await validateUnwrapWithLaya(finding, mockPi);
  assert.equal(calledTool, "mcp__laya_predict");
  assert.equal(isSafe, true);
});

test("compatibility: works with Pi host environment", async () => {
  const tools: Record<string, any> = {};
  const commands: Record<string, any> = {};
  const handlers: Record<string, Function[]> = {};

  const piHost = {
    registerTool: (tool: any) => {
      tools[tool.name] = tool;
    },
    registerCommand: (name: string, desc: string, handler: Function) => {
      commands[name] = { desc, handler };
    },
    on: (event: string, handler: Function) => {
      handlers[event] = handlers[event] || [];
      handlers[event].push(handler);
    },
  };

  registerExtension(piHost as any);

  // 1. Tool registration
  assert.ok(tools["habit_hooks"]);
  assert.ok(tools["habit_snooze"]);

  // 2. Pi execute calling convention
  const result = await tools["habit_hooks"].execute(
    "tool-call-1",
    { scope: "working" },
    undefined,
    undefined,
    { cwd: process.cwd() }
  );
  assert.equal(typeof result.exitCode, "number");
  assert.equal(typeof result.passed, "boolean");
  assert.ok(Array.isArray(result.content));

  // 3. Command registration
  assert.ok(commands["habit-hooks"]);
  assert.ok(commands["snooze"]);

  // 4. Prompt injection via before_agent_start
  assert.ok(handlers["before_agent_start"]);
  const promptEvt = handlers["before_agent_start"][0]({ systemPrompt: ["Base prompt"] });
  assert.ok(
    Array.isArray(promptEvt.systemPrompt) &&
      promptEvt.systemPrompt.some((p: string) => p.includes("Habit Hooks Quality Coaching Rules"))
  );

  // 5. Lifecycle event hooks
  assert.ok(handlers["tool_call"]);
  assert.ok(handlers["turn_end"]);
});

test("compatibility: works with Oh My Pi (OMP) host environment", async () => {
  const tools: Record<string, any> = {};
  const commands: Record<string, any> = {};
  const handlers: Record<string, Function[]> = {};
  let injectedPrompt = "";
  const statusUpdates: Array<{ key: string; status: string | { text: string } | undefined }> = [];

  const ompHost = {
    registerTool: (tool: any) => {
      tools[tool.name] = tool;
    },
    registerCommand: (cmdObj: { name: string; description: string; handler: Function }) => {
      commands[cmdObj.name] = cmdObj;
    },
    injectSystemPrompt: (prompt: string) => {
      injectedPrompt = prompt;
    },
    ui: {
      setStatus: (key: string, status: string | { text: string } | undefined) => {
        statusUpdates.push({ key, status });
      },
    },
    on: (event: string, handler: Function) => {
      handlers[event] = handlers[event] || [];
      handlers[event].push(handler);
    },
  };

  registerExtension(ompHost as any);

  // 1. Tool registration
  assert.ok(tools["habit_hooks"]);
  assert.ok(tools["habit_snooze"]);

  // 2. OMP handler calling convention
  const result = await tools["habit_hooks"].handler(
    { scope: "working" },
    { cwd: process.cwd() }
  );
  assert.equal(typeof result.exitCode, "number");
  assert.equal(typeof result.passed, "boolean");
  assert.ok(Array.isArray(result.content));

  // 3. Direct command object registration
  assert.ok(commands["habit-hooks"]);
  assert.ok(commands["snooze"]);

  // 4. Direct system prompt injection
  assert.ok(injectedPrompt.includes("Habit Hooks Quality Coaching Rules"));

  // 5. OMP before_yield and session_stop event hooks
  assert.ok(handlers["before_yield"]);
  assert.ok(handlers["session_stop"]);
  const stopResult = await handlers["session_stop"][0]({});
  assert.equal(typeof stopResult.interrupt, "boolean");
  if (stopResult.interrupt) {
    assert.equal(stopResult.decision, "block");
    assert.ok(stopResult.reason.includes("Habit Hooks Quality Gate"));
  }

  // 6. OMP tool_call hook with args parameter shape
  assert.ok(handlers["tool_call"]);
  const toolCallRes = await handlers["tool_call"][0]({
    toolName: "edit",
    input: { path: "crates/habit-core/src/lib.rs" },
  });
  assert.ok(statusUpdates.length > 0);
  assert.equal(statusUpdates[0].key, "habit-hooks");
});
