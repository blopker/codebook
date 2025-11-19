"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const path = __importStar(require("node:path"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
const binary_1 = require("./binary");
const languages_1 = require("./languages");
const DEFAULT_CLIENT_KEY = "__default__";
let outputChannel;
let contextRef;
let binaryManager;
let refreshPromise = Promise.resolve();
const clients = new Map();
async function activate(context) {
    contextRef = context;
    outputChannel = vscode.window.createOutputChannel("Codebook");
    context.subscriptions.push(outputChannel);
    binaryManager = new binary_1.CodebookBinaryManager(context, outputChannel);
    context.subscriptions.push(vscode.commands.registerCommand("codebook.restart", async () => {
        vscode.window.showInformationMessage("Restarting Codebook language server…");
        try {
            await queueRefresh();
            vscode.window.showInformationMessage("Codebook language server restarted.");
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to restart Codebook: ${error}`);
        }
    }));
    context.subscriptions.push(vscode.workspace.onDidChangeWorkspaceFolders(async () => {
        await queueRefresh();
    }));
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration(async (event) => {
        const invalidateCache = event.affectsConfiguration("codebook.binaryPath") ||
            event.affectsConfiguration("codebook.enablePrerelease");
        const forceDownload = event.affectsConfiguration("codebook.enablePrerelease");
        if (event.affectsConfiguration("codebook.binaryPath") ||
            event.affectsConfiguration("codebook.enablePrerelease") ||
            event.affectsConfiguration("codebook.logLevel")) {
            await queueRefresh({ forceDownload, invalidateCache });
        }
    }));
    await queueRefresh();
}
async function queueRefresh(options = {}) {
    refreshPromise = refreshPromise.then(() => refreshClients(options));
    await refreshPromise;
}
async function refreshClients(options = {}) {
    await stopAllClients();
    if (options.invalidateCache) {
        await binaryManager.invalidateCache();
    }
    try {
        const binaryPath = await binaryManager.getBinaryPath(options.forceDownload);
        const folders = vscode.workspace.workspaceFolders;
        if (!folders || folders.length === 0) {
            await startClient(undefined, binaryPath);
        }
        else {
            for (const folder of folders) {
                await startClient(folder, binaryPath);
            }
        }
    }
    catch (error) {
        outputChannel.appendLine(String(error));
        vscode.window.showErrorMessage(String(error));
    }
}
async function startClient(folder, binaryPath) {
    const key = folder?.uri.toString() ?? DEFAULT_CLIENT_KEY;
    if (clients.has(key)) {
        return;
    }
    const root = determineRoot(folder);
    const config = vscode.workspace.getConfiguration("codebook", folder);
    const logLevel = config.get("logLevel", "info");
    const args = [`--root=${root}`, "serve"];
    outputChannel.appendLine(`Starting Codebook (${args.join(" ")}) for ${folder?.name ?? root}`);
    const serverOptions = {
        run: {
            command: binaryPath,
            args,
            options: {
                env: {
                    ...process.env,
                    RUST_LOG: logLevel,
                },
            },
        },
        debug: {
            command: binaryPath,
            args,
            options: {
                env: {
                    ...process.env,
                    RUST_LOG: "debug",
                },
            },
        },
    };
    const clientOptions = {
        documentSelector: languages_1.DOCUMENT_SELECTOR,
        diagnosticCollectionName: "codebook",
        outputChannel,
        workspaceFolder: folder,
    };
    const client = new node_1.LanguageClient(`codebook-${key}`, "Codebook", serverOptions, clientOptions);
    clients.set(key, client);
    contextRef.subscriptions.push(client);
    await client.start();
}
function determineRoot(folder) {
    if (folder) {
        return folder.uri.fsPath;
    }
    const editor = vscode.window.activeTextEditor;
    if (editor && editor.document.uri.scheme === "file") {
        return path.dirname(editor.document.uri.fsPath);
    }
    const defaultWorkspace = vscode.workspace.workspaceFolders?.[0];
    if (defaultWorkspace) {
        return defaultWorkspace.uri.fsPath;
    }
    return process.cwd();
}
async function stopAllClients() {
    const runningClients = Array.from(clients.values());
    clients.clear();
    await Promise.allSettled(runningClients.map((client) => client.stop()));
}
async function deactivate() {
    await stopAllClients();
}
//# sourceMappingURL=extension.js.map