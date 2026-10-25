export interface ToolContent {
    type: "text";
    text: string;
}
export interface ToolResult {
    content: ToolContent[];
    details?: Record<string, unknown>;
    exitCode?: number;
    passed?: boolean;
    guidance?: string;
    output?: string;
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
    registerTool: (tool: Record<string, unknown>) => void;
    registerCommand?: (...args: unknown[]) => void;
    injectSystemPrompt?: (prompt: string) => void;
    on?: (event: string, handler: (data: {
        systemPrompt?: string | string[];
    }) => {
        systemPrompt?: string | string[];
    } | void) => void;
}
export default function (pi: PiAgentHost): void;
