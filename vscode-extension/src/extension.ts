import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import * as vscode from "vscode";
import {
  LanguageClient,
  type LanguageClientOptions,
  RevealOutputChannelOn,
  type ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

const EXTENSION_LSP_NAME = "codebook-lsp";
const GITHUB_REPO = "blopker/codebook";

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel | undefined;

interface GitHubRelease {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  tag_name: string;
  assets: Array<{
    name: string;
    // eslint-disable-next-line @typescript-eslint/naming-convention
    browser_download_url: string;
  }>;
}

interface BinaryInfo {
  path: string;
  version: string;
}

function getPlatformInfo(): { platform: string; arch: string; ext: string } {
  const platform = os.platform();
  const arch = os.arch();

  let platformStr: string;
  let archStr: string;
  let ext: string;

  // Map Node.js arch to Rust target arch
  switch (arch) {
    case "x64":
      archStr = "x86_64";
      break;
    case "arm64":
      archStr = "aarch64";
      break;
    default:
      throw new Error(`Unsupported architecture: ${arch}`);
  }

  // Map Node.js platform to Rust target platform
  switch (platform) {
    case "darwin":
      platformStr = "apple-darwin";
      ext = "tar.gz";
      break;
    case "linux":
      platformStr = "unknown-linux-musl";
      ext = "tar.gz";
      break;
    case "win32":
      platformStr = "pc-windows-msvc";
      ext = "zip";
      break;
    default:
      throw new Error(`Unsupported platform: ${platform}`);
  }

  return { platform: platformStr, arch: archStr, ext };
}

async function fetchLatestRelease(): Promise<GitHubRelease> {
  const url = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;

  try {
    const response = await fetch(url, {
      headers: {
        // eslint-disable-next-line @typescript-eslint/naming-convention
        "User-Agent": "vscode-codebook-extension",
        // eslint-disable-next-line @typescript-eslint/naming-convention
        Accept: "application/vnd.github.v3+json",
      },
    });

    if (!response.ok) {
      throw new Error(
        `GitHub API returned ${response.status}: ${response.statusText}`,
      );
    }

    return (await response.json()) as GitHubRelease;
  } catch (error) {
    throw new Error(`Failed to fetch latest release: ${error}`);
  }
}

async function downloadFile(
  url: string,
  destPath: string,
  progressCallback?: (progress: number) => void,
): Promise<void> {
  const response = await fetch(url, {
    headers: {
      // eslint-disable-next-line @typescript-eslint/naming-convention
      "User-Agent": "vscode-codebook-extension",
    },
  });

  if (!response.ok) {
    throw new Error(
      `Failed to download: ${response.status} ${response.statusText}`,
    );
  }

  const contentLength = response.headers.get("content-length");
  const totalSize = contentLength ? parseInt(contentLength, 10) : 0;

  const destDir = path.dirname(destPath);
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  const fileStream = fs.createWriteStream(destPath);
  const reader = response.body?.getReader();

  if (!reader) {
    throw new Error("Failed to get response body reader");
  }

  let downloadedSize = 0;

  try {
    // eslint-disable-next-line no-constant-condition
    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }

      fileStream.write(value);
      downloadedSize += value.length;

      if (progressCallback && totalSize > 0) {
        const progress = (downloadedSize / totalSize) * 100;
        progressCallback(progress);
      }
    }
  } finally {
    fileStream.close();
  }
}

async function extractArchive(
  archivePath: string,
  destDir: string,
  ext: string,
): Promise<void> {
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  if (ext === "tar.gz") {
    // Dynamic import for tar module
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const tar = require("tar");
    await tar.extract({
      file: archivePath,
      cwd: destDir,
    });
  } else if (ext === "zip") {
    // Dynamic import for adm-zip module
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const admZip = require("adm-zip");
    const zip = new admZip(archivePath);
    zip.extractAllTo(destDir, true);
  } else {
    throw new Error(`Unsupported archive format: ${ext}`);
  }
}

function makeExecutable(filePath: string): void {
  if (os.platform() !== "win32") {
    fs.chmodSync(filePath, 0o755);
  }
}

function getCacheDir(context: vscode.ExtensionContext): string {
  return path.join(context.globalStorageUri.fsPath, "binaries");
}

function getCachedBinary(context: vscode.ExtensionContext): BinaryInfo | null {
  const cacheDir = getCacheDir(context);
  const metaPath = path.join(cacheDir, "binary-meta.json");

  if (!fs.existsSync(metaPath)) {
    return null;
  }

  try {
    const meta = JSON.parse(fs.readFileSync(metaPath, "utf8"));
    const binaryPath = meta.path;

    if (fs.existsSync(binaryPath)) {
      return meta as BinaryInfo;
    }
  } catch (error) {
    outputChannel?.appendLine(
      `Failed to read cached binary metadata: ${error}`,
    );
  }

  return null;
}

function saveBinaryMeta(
  context: vscode.ExtensionContext,
  info: BinaryInfo,
): void {
  const cacheDir = getCacheDir(context);
  const metaPath = path.join(cacheDir, "binary-meta.json");

  if (!fs.existsSync(cacheDir)) {
    fs.mkdirSync(cacheDir, { recursive: true });
  }

  fs.writeFileSync(metaPath, JSON.stringify(info, null, 2));
}

function resolveGlobalConfigPath(
  rawPath: string | undefined,
): string | undefined {
  if (!rawPath) {
    return undefined;
  }

  const trimmed = rawPath.trim();
  if (!trimmed) {
    return undefined;
  }

  let expanded = trimmed;
  if (expanded.startsWith("~")) {
    const home = os.homedir();
    expanded = path.join(home, expanded.slice(1));
  }

  if (!path.isAbsolute(expanded)) {
    expanded = path.resolve(expanded);
  }

  return expanded;
}

async function findLocalBinary(): Promise<string | null> {
  // Check if binary exists in PATH
  const whichCommand = process.platform === "win32" ? "where" : "which";
  try {
    const result = execSync(`${whichCommand} ${EXTENSION_LSP_NAME}`, {
      encoding: "utf8",
    });
    return result.trim().split("\n")[0];
  } catch {
    // Not found in PATH
  }

  // Check cargo install location
  const cargoPath = path.join(
    process.env.HOME || process.env.USERPROFILE || "",
    ".cargo",
    "bin",
    EXTENSION_LSP_NAME + (process.platform === "win32" ? ".exe" : ""),
  );

  if (fs.existsSync(cargoPath)) {
    return cargoPath;
  }

  return null;
}

async function downloadAndInstallBinary(
  context: vscode.ExtensionContext,
  progressCallback?: (message: string) => void,
): Promise<BinaryInfo> {
  const { platform, arch, ext } = getPlatformInfo();

  progressCallback?.("Fetching latest release information...");
  const release = await fetchLatestRelease();

  const assetName = `${EXTENSION_LSP_NAME}-${arch}-${platform}.${ext}`;
  const asset = release.assets.find((a) => a.name === assetName);

  if (!asset) {
    throw new Error(`No compatible binary found for ${arch}-${platform}`);
  }

  const cacheDir = getCacheDir(context);
  const versionDir = path.join(cacheDir, release.tag_name);
  const binaryName =
    EXTENSION_LSP_NAME + (process.platform === "win32" ? ".exe" : "");
  const binaryPath = path.join(versionDir, binaryName);

  // Check if this version is already downloaded
  if (fs.existsSync(binaryPath)) {
    return { path: binaryPath, version: release.tag_name };
  }

  // Clean up old versions
  if (fs.existsSync(cacheDir)) {
    const dirs = fs.readdirSync(cacheDir).filter((d) => {
      const dirPath = path.join(cacheDir, d);
      return fs.statSync(dirPath).isDirectory() && d !== release.tag_name;
    });

    for (const dir of dirs) {
      try {
        fs.rmSync(path.join(cacheDir, dir), { recursive: true, force: true });
      } catch (error) {
        outputChannel?.appendLine(
          `Failed to remove old version ${dir}: ${error}`,
        );
      }
    }
  }

  // Download the binary
  const tempDir = path.join(cacheDir, `temp-${Date.now()}`);
  const archivePath = path.join(tempDir, assetName);

  try {
    progressCallback?.(`Downloading ${assetName}...`);

    await downloadFile(asset.browser_download_url, archivePath, (progress) => {
      progressCallback?.(`Downloading: ${Math.round(progress)}%`);
    });

    progressCallback?.("Extracting archive...");
    await extractArchive(archivePath, versionDir, ext);

    // Make binary executable
    makeExecutable(binaryPath);

    // Clean up temp directory
    fs.rmSync(tempDir, { recursive: true, force: true });

    const info = { path: binaryPath, version: release.tag_name };
    saveBinaryMeta(context, info);

    return info;
  } catch (error) {
    // Clean up on error
    try {
      if (fs.existsSync(tempDir)) {
        fs.rmSync(tempDir, { recursive: true, force: true });
      }
      if (fs.existsSync(versionDir)) {
        fs.rmSync(versionDir, { recursive: true, force: true });
      }
    } catch {
      // Ignore cleanup errors
    }

    throw error;
  }
}

async function getBinary(context: vscode.ExtensionContext): Promise<string> {
  const config = vscode.workspace.getConfiguration("codebook");

  // Check if user has specified a custom path
  const customPath = config.get<string>("serverPath");
  if (customPath && customPath !== EXTENSION_LSP_NAME) {
    if (path.isAbsolute(customPath) && fs.existsSync(customPath)) {
      outputChannel?.appendLine(`Using custom binary path: ${customPath}`);
      return customPath;
    }
  }

  // Check for local development binary
  const devPath = path.join(
    context.extensionPath,
    "..",
    "..",
    "target",
    "release",
    EXTENSION_LSP_NAME,
  );
  if (fs.existsSync(devPath)) {
    outputChannel?.appendLine(`Using development binary: ${devPath}`);
    return devPath;
  }

  // Check for locally installed binary
  const localBinary = await findLocalBinary();
  if (localBinary) {
    outputChannel?.appendLine(`Using local binary: ${localBinary}`);
    return localBinary;
  }

  // Check for cached binary
  const cached = getCachedBinary(context);
  if (cached) {
    outputChannel?.appendLine(
      `Using cached binary: ${cached.path} (version: ${cached.version})`,
    );
    return cached.path;
  }

  // Download and install binary
  outputChannel?.appendLine(
    "No local binary found, downloading from GitHub...",
  );

  const info = await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "Codebook",
      cancellable: false,
    },
    async (progress) => {
      return downloadAndInstallBinary(context, (message) => {
        progress.report({ message });
      });
    },
  );

  outputChannel?.appendLine(
    `Downloaded and installed binary: ${info.path} (version: ${info.version})`,
  );
  return info.path;
}

export async function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("codebook");

  if (!config.get<boolean>("enable", true)) {
    return;
  }

  outputChannel = vscode.window.createOutputChannel("Codebook");

  try {
    const serverPath = await getBinary(context);
    const resolvedGlobalConfigPath = resolveGlobalConfigPath(
      config.get<string>("globalConfigPath", ""),
    );

    // Server options
    const serverOptions: ServerOptions = {
      command: serverPath,
      args: ["--root=./", "serve"],
      transport: TransportKind.stdio,
    };

    // Client options
    const initializationOptions: Record<string, unknown> = {
      logLevel: config.get<string>("logLevel", "info"),
    };
    if (resolvedGlobalConfigPath) {
      initializationOptions.globalConfigPath = resolvedGlobalConfigPath;
    }

    const clientOptions: LanguageClientOptions = {
      documentSelector: [
        { scheme: "file", language: "*" },
        { scheme: "untitled", language: "*" },
      ],
      outputChannel,
      revealOutputChannelOn: RevealOutputChannelOn.Error,
      initializationOptions,
    };

    // Create the language client
    client = new LanguageClient(
      "codebook",
      "Codebook Language Server",
      serverOptions,
      clientOptions,
    );

    // Register commands
    context.subscriptions.push(
      vscode.commands.registerCommand("codebook.restart", async () => {
        if (client) {
          await client.stop();
          await client.start();
          vscode.window.showInformationMessage(
            "Codebook language server restarted",
          );
        }
      }),
    );

    context.subscriptions.push(
      vscode.commands.registerCommand("codebook.showOutputChannel", () => {
        outputChannel?.show();
      }),
    );

    // Start the client
    await client.start();
    outputChannel.appendLine("Codebook language server started successfully");
  } catch (error) {
    const errorMessage = `Failed to start Codebook language server: ${error}`;
    outputChannel?.appendLine(errorMessage);
    vscode.window.showErrorMessage(errorMessage);

    // Suggest manual installation
    const selection = await vscode.window.showInformationMessage(
      "Would you like to manually install the Codebook language server?",
      "Install Instructions",
    );

    if (selection === "Install Instructions") {
      vscode.env.openExternal(
        vscode.Uri.parse(`https://github.com/${GITHUB_REPO}#installation`),
      );
    }
  }
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
