// Minimal stub for VS Code API
// This will be loaded into the Deno runtime

globalThis.vscode = {
    window: {
        showInformationMessage: (message) => {
            Deno.core.print(`[ExtHost] Info: ${message}\n`);
            // In a real implementation, this would send an IPC message to the host
            Deno.core.ops.op_show_info(message);
        },
        showErrorMessage: (message) => {
            Deno.core.print(`[ExtHost] Error: ${message}\n`);
        }
    },
    workspace: {
        rootPath: null,
    }
};
