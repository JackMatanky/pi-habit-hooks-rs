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
    callTool?: (toolName: string, args: Record<string, unknown>) => Promise<unknown>;
    ui?: {
        setStatus?: (key: string, status: {
            text: string;
        }) => void;
        log?: (msg: string) => void;
    };
    on?: (event: string, handler: (data: any, ctx?: any) => any) => void;
}
export declare function validateUnwrapWithLaya(finding: {
    smell: string;
    issues?: Array<{
        details?: {
            content?: string;
        };
    }>;
}, pi: PiAgentHost): Promise<boolean | undefined>;
export default function (pi: PiAgentHost): void;
