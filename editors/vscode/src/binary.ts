import * as vscode from "vscode";
import * as fs from "node:fs";
import * as fsp from "node:fs/promises";
import * as path from "node:path";

const BINARY_BASENAME = "codebook-lsp";
const BINARY_FILENAME =
  process.platform === "win32" ? `${BINARY_BASENAME}.exe` : BINARY_BASENAME;

export class CodebookBinaryManager {
  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly logger: vscode.OutputChannel,
  ) {}

  async getBinaryPath(): Promise<string> {
    const config = vscode.workspace.getConfiguration("codebook");
    const explicitPath = config.get<string>("binaryPath")?.trim();
    if (explicitPath) {
      await ensureExecutable(explicitPath);
      this.logger.appendLine(
        `Using codebook binary from configuration: ${explicitPath}`,
      );
      return explicitPath;
    }

    const systemBinary = await findOnPath();
    if (systemBinary) {
      this.logger.appendLine(
        `Using codebook binary from PATH: ${systemBinary}`,
      );
      return systemBinary;
    }

    const bundled = path.join(
      this.context.extensionPath,
      "bin",
      BINARY_FILENAME,
    );
    if (await fileExists(bundled)) {
      if (process.platform !== "win32") {
        // The vsix archive may strip the executable bit on extraction.
        await fsp.chmod(bundled, 0o755).catch(() => {});
      }
      this.logger.appendLine(`Using bundled codebook binary: ${bundled}`);
      return bundled;
    }

    throw new Error(
      "No codebook-lsp binary was found. Install codebook-lsp on your PATH or set codebook.binaryPath.",
    );
  }
}

async function ensureExecutable(filePath: string): Promise<void> {
  try {
    await fsp.access(filePath, fs.constants.X_OK);
  } catch {
    throw new Error(
      `The configured codebook binary is not executable: ${filePath}`,
    );
  }
}

async function findOnPath(): Promise<string | undefined> {
  const pathEntries = (process.env.PATH ?? "").split(path.delimiter);
  const extensions =
    process.platform === "win32"
      ? (process.env.PATHEXT ?? ".EXE;.CMD;.BAT;.COM").split(";")
      : [""];

  for (const entry of pathEntries) {
    for (const extension of extensions) {
      const candidate = path.join(entry, `${BINARY_BASENAME}${extension}`);
      if (await isExecutable(candidate)) {
        return candidate;
      }
    }
  }

  return undefined;
}

async function isExecutable(filePath: string): Promise<boolean> {
  try {
    await fsp.access(
      filePath,
      process.platform === "win32" ? fs.constants.F_OK : fs.constants.X_OK,
    );
    return true;
  } catch {
    return false;
  }
}

async function fileExists(filePath: string): Promise<boolean> {
  try {
    await fsp.access(filePath, fs.constants.F_OK);
    return true;
  } catch {
    return false;
  }
}
