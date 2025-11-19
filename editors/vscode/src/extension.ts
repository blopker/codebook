import * as path from "node:path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { CodebookBinaryManager } from "./binary";
import { DOCUMENT_SELECTOR } from "./languages";

const DEFAULT_CLIENT_KEY = "__default__";

let outputChannel: vscode.OutputChannel;
let contextRef: vscode.ExtensionContext;
let binaryManager: CodebookBinaryManager;
let refreshPromise: Promise<void> = Promise.resolve();

const clients = new Map<string, LanguageClient>();

export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  contextRef = context;
  outputChannel = vscode.window.createOutputChannel("Codebook");
  context.subscriptions.push(outputChannel);

  binaryManager = new CodebookBinaryManager(context, outputChannel);

  context.subscriptions.push(
    vscode.commands.registerCommand("codebook.restart", async () => {
      vscode.window.showInformationMessage(
        "Restarting Codebook language server…",
      );
      try {
        await queueRefresh();
        vscode.window.showInformationMessage(
          "Codebook language server restarted.",
        );
      } catch (error) {
        vscode.window.showErrorMessage(`Failed to restart Codebook: ${error}`);
      }
    }),
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeWorkspaceFolders(async () => {
      await queueRefresh();
    }),
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      const invalidateCache =
        event.affectsConfiguration("codebook.binaryPath") ||
        event.affectsConfiguration("codebook.enablePrerelease");
      const forceDownload = event.affectsConfiguration(
        "codebook.enablePrerelease",
      );

      if (
        event.affectsConfiguration("codebook.binaryPath") ||
        event.affectsConfiguration("codebook.enablePrerelease") ||
        event.affectsConfiguration("codebook.logLevel")
      ) {
        await queueRefresh({ forceDownload, invalidateCache });
      }
    }),
  );

  await queueRefresh();
}

type RefreshOptions = {
  forceDownload?: boolean;
  invalidateCache?: boolean;
};

async function queueRefresh(options: RefreshOptions = {}): Promise<void> {
  refreshPromise = refreshPromise.then(() => refreshClients(options));
  await refreshPromise;
}

async function refreshClients(options: RefreshOptions = {}): Promise<void> {
  await stopAllClients();

  if (options.invalidateCache) {
    await binaryManager.invalidateCache();
  }

  try {
    const binaryPath = await binaryManager.getBinaryPath(options.forceDownload);
    const folders = vscode.workspace.workspaceFolders;
    if (!folders || folders.length === 0) {
      await startClient(undefined, binaryPath);
    } else {
      for (const folder of folders) {
        await startClient(folder, binaryPath);
      }
    }
  } catch (error) {
    outputChannel.appendLine(String(error));
    vscode.window.showErrorMessage(String(error));
  }
}

async function startClient(
  folder: vscode.WorkspaceFolder | undefined,
  binaryPath: string,
): Promise<void> {
  const key = folder?.uri.toString() ?? DEFAULT_CLIENT_KEY;
  if (clients.has(key)) {
    return;
  }

  const root = determineRoot(folder);
  const config = vscode.workspace.getConfiguration("codebook", folder);
  const logLevel = config.get<string>("logLevel", "info");

  const args = [`--root=${root}`, "serve"];
  outputChannel.appendLine(
    `Starting Codebook (${args.join(" ")}) for ${folder?.name ?? root}`,
  );

  const serverOptions: ServerOptions = {
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

  const clientOptions: LanguageClientOptions = {
    documentSelector: DOCUMENT_SELECTOR,
    diagnosticCollectionName: "codebook",
    outputChannel,
    workspaceFolder: folder,
  };

  const client = new LanguageClient(
    `codebook-${key}`,
    "Codebook",
    serverOptions,
    clientOptions,
  );
  clients.set(key, client);
  contextRef.subscriptions.push(client);
  await client.start();
}

function determineRoot(folder: vscode.WorkspaceFolder | undefined): string {
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

async function stopAllClients(): Promise<void> {
  const runningClients = Array.from(clients.values());
  clients.clear();
  await Promise.allSettled(runningClients.map((client) => client.stop()));
}

export async function deactivate(): Promise<void> {
  await stopAllClients();
}
