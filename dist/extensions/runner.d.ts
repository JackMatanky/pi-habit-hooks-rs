export declare function resolveBinary(name: "habit-hooks" | "habit-snooze" | "habit-sensors" | "habit-mapper"): string;
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
export declare function runHabitHooks(options?: RunHabitHooksOptions): HabitHooksResult;
export interface RunHabitSnoozeOptions {
    action: "list" | "add" | "prune";
    reason?: string;
    cwd?: string;
    inputJson?: string;
}
export declare function runHabitSnooze(options: RunHabitSnoozeOptions): {
    exitCode: number;
    output: string;
};
