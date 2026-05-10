import * as path from "node:path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { CodebookBinaryManager } from "./binary";
import { DOCUMENT_SELECTOR } from "./languages";

let outputChannel: vscode.OutputChannel;
let contextRef: vscode.ExtensionContext;
let binaryManager: CodebookBinaryManager;
let refreshPromise: Promise<void> = Promise.resolve();

let client: LanguageClient | undefined;

export async function activate(
  context: vscode.ExtensionContext
): Promise<void> {
  contextRef = context;
  outputChannel = vscode.window.createOutputChannel("Codebook");
  context.subscriptions.push(outputChannel);

  binaryManager = new CodebookBinaryManager(context, outputChannel);

  context.subscriptions.push(
    vscode.commands.registerCommand("codebook.restart", async () => {
      vscode.window.showInformationMessage(
        "Restarting Codebook language server…"
      );
      try {
        await queueRefresh();
        vscode.window.showInformationMessage(
          "Codebook language server restarted."
        );
      } catch (error) {
        vscode.window.showErrorMessage(`Failed to restart Codebook: ${error}`);
      }
    })
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeWorkspaceFolders(async () => {
      await queueRefresh();
    })
  );

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (
        event.affectsConfiguration("codebook.binaryPath") ||
        event.affectsConfiguration("codebook.logLevel")
      ) {
        await queueRefresh();
      }
    })
  );

  await queueRefresh();
}

async function queueRefresh(): Promise<void> {
  refreshPromise = refreshPromise.then(() => refreshClients());
  await refreshPromise;
}

async function refreshClients(): Promise<void> {
  await stopClient();

  try {
    const binaryPath = await binaryManager.getBinaryPath();
    await startClient(binaryPath);
  } catch (error) {
    outputChannel.appendLine(String(error));
    vscode.window.showErrorMessage(String(error));
  }
}

async function startClient(binaryPath: string): Promise<void> {
  if (client) {
    return;
  }

  const folders = vscode.workspace.workspaceFolders;
  const primaryFolder = folders?.[0];
  const root = determineRoot(primaryFolder);
  const config = vscode.workspace.getConfiguration("codebook", primaryFolder);
  const logLevel = config.get<string>("logLevel", "info");

  const args = [`--root=${root}`, "serve"];
  outputChannel.appendLine(
    `Starting Codebook (${args.join(" ")}) for ${primaryFolder?.name ?? root}`
  );
  if (folders && folders.length > 1) {
    outputChannel.appendLine(
      `Multi-root workspace detected; using ${primaryFolder?.name ?? root} as the project config root. Per-folder codebook.toml files in other folders are not yet honored.`
    );
  }

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
  };

  client = new LanguageClient(
    "codebook",
    "Codebook",
    serverOptions,
    clientOptions
  );
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

  return process.cwd();
}

async function stopClient(): Promise<void> {
  const running = client;
  client = undefined;
  if (running) {
    await running.stop().catch((error) => {
      outputChannel.appendLine(`Error stopping Codebook client: ${error}`);
    });
  }
}

export async function deactivate(): Promise<void> {
  await stopClient();
}
