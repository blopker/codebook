import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import {
  LanguageClient,
  type LanguageClientOptions,
  RevealOutputChannelOn,
  type ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("codebook");

  if (!config.get<boolean>("enable", true)) {
    return;
  }

  const outputChannel = vscode.window.createOutputChannel("Codebook");

  // Get the server path from configuration or use default
  // let serverPath = config.get<string>("serverPath", "codebook-lsp");
  let serverPath = "/Users/blopker/code/codebook/target/release/codebook-lsp";

  // If the server path is not absolute, try to find it in PATH
  if (!path.isAbsolute(serverPath)) {
    const whichCommand = process.platform === "win32" ? "where" : "which";
    try {
      const result = execSync(`${whichCommand} ${serverPath}`, {
        encoding: "utf8",
      });
      serverPath = result.trim().split("\n")[0];
    } catch (_) {
      // If not found in PATH, check if it's installed via cargo
      const cargoPath = path.join(
        process.env.HOME || process.env.USERPROFILE || "",
        ".cargo",
        "bin",
        serverPath
      );
      if (fs.existsSync(cargoPath)) {
        serverPath = cargoPath;
      } else {
        vscode.window.showErrorMessage(
          `Codebook LSP server not found. Please install it with: cargo install codebook-lsp`
        );
        return;
      }
    }
  }

  // Verify the server executable exists
  if (!fs.existsSync(serverPath)) {
    vscode.window.showErrorMessage(
      `Codebook LSP server not found at ${serverPath}. Please install it with: cargo install codebook-lsp`
    );
    return;
  }

  // Server options
  const serverOptions: ServerOptions = {
    command: serverPath,
    args: [`--root=./`, "serve"],
    transport: TransportKind.stdio,
  };

  // Client options
  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "*" },
      { scheme: "untitled", language: "*" },
    ],
    outputChannel,
    revealOutputChannelOn: RevealOutputChannelOn.Error,
  };

  // Create the language client
  client = new LanguageClient(
    "codebook",
    "Codebook Language Server",
    serverOptions,
    clientOptions
  );

  // Start the client
  try {
    await client.start();
    outputChannel.appendLine("Codebook language server started successfully");
  } catch (error) {
    outputChannel.appendLine(
      `Failed to start Codebook language server: ${error}`
    );
    vscode.window.showErrorMessage(
      `Failed to start Codebook language server: ${error}`
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
