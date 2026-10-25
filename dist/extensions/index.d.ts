export interface ToolDefinition {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
    handler: (args: Record<string, unknown>, context?: ExtensionContext) => Promise<unknown>;
}
export interface CommandDefinition {
    name: string;
    description: string;
    handler: (args: string, context?: ExtensionContext) => Promise<void>;
}
export interface ExtensionContext {
    cwd?: string;
    ui?: {
        setStatus?: (key: string, status: {
            text: string;
        }) => void;
        log?: (msg: string) => void;
    };
}
export interface PiAgentHost {
    registerTool: (tool: ToolDefinition) => void;
    registerCommand?: (command: CommandDefinition) => void;
    injectSystemPrompt?: (prompt: string) => void;
}
export default function (pi: PiAgentHost): void;
