import * as vscode from "vscode";
import * as fs from "node:fs";
import * as fsp from "node:fs/promises";
import * as path from "node:path";
import * as os from "node:os";
import * as https from "node:https";
import type { IncomingMessage } from "node:http";
import { pipeline } from "node:stream/promises";
import AdmZip from "adm-zip";
import tar from "tar";
import which from "which";

const BINARY_BASENAME = "codebook-lsp";
const BINARY_FILENAME =
  process.platform === "win32" ? `${BINARY_BASENAME}.exe` : BINARY_BASENAME;
const VERSION_FILENAME = "codebook.version";
const USER_AGENT = "codebook-vscode-extension";
const MAX_REDIRECTS = 5;

interface GithubAsset {
  name: string;
  browser_download_url: string;
}

interface GithubRelease {
  id: number;
  tag_name?: string;
  name?: string;
  prerelease?: boolean;
  assets: GithubAsset[];
}

type ArchiveType = "tar-gz" | "zip";

export class CodebookBinaryManager {
  private binaryPath?: string;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly logger: vscode.OutputChannel,
  ) {}

  async getBinaryPath(forceRedownload = false): Promise<string> {
    if (this.binaryPath && !forceRedownload) {
      return this.binaryPath;
    }

    const config = vscode.workspace.getConfiguration("codebook");
    const explicitPath = config.get<string>("binaryPath")?.trim();
    if (explicitPath) {
      await ensureExecutable(explicitPath);
      this.logger.appendLine(
        `Using codebook binary from configuration: ${explicitPath}`,
      );
      this.binaryPath = explicitPath;
      return explicitPath;
    }

    if (!forceRedownload) {
      const systemBinary = await findOnPath();
      if (systemBinary) {
        this.logger.appendLine(
          `Using codebook binary from PATH: ${systemBinary}`,
        );
        this.binaryPath = systemBinary;
        return systemBinary;
      }
    }

    const storageDir = await this.ensureStorageDir();
    const cached = await this.getCachedBinary(storageDir);
    const enablePrerelease = config.get<boolean>("enablePrerelease", false);

    try {
      const release = await fetchDesiredRelease(enablePrerelease);
      const releaseVersion = extractReleaseVersion(release);

      if (
        !forceRedownload &&
        cached &&
        cached.version === releaseVersion &&
        (await fileExists(cached.path))
      ) {
        this.logger.appendLine(
          `Using cached codebook binary version ${releaseVersion}`,
        );
        this.binaryPath = cached.path;
        return cached.path;
      }

      const binaryPath = await this.downloadBinary(
        storageDir,
        release,
        releaseVersion,
      );
      this.binaryPath = binaryPath;
      return binaryPath;
    } catch (error) {
      if (cached && (await fileExists(cached.path))) {
        this.logger.appendLine(
          `Failed to update Codebook (${error}). Falling back to cached binary at ${cached.path}`,
        );
        this.binaryPath = cached.path;
        return cached.path;
      }
      throw error;
    }
  }

  async invalidateCache(): Promise<void> {
    this.binaryPath = undefined;
  }

  private async ensureStorageDir(): Promise<string> {
    const storageDir = this.context.globalStorageUri.fsPath;
    await fsp.mkdir(storageDir, { recursive: true });
    return storageDir;
  }

  private async getCachedBinary(
    storageDir: string,
  ): Promise<{ path: string; version: string } | undefined> {
    try {
      const versionFile = path.join(storageDir, VERSION_FILENAME);
      const version = (await fsp.readFile(versionFile, "utf8")).trim();
      if (!version) {
        return undefined;
      }

      const binaryPath = path.join(
        storageDir,
        versionDirectoryName(version),
        BINARY_FILENAME,
      );
      if (await fileExists(binaryPath)) {
        return { path: binaryPath, version };
      }
    } catch {
      // ignore missing cache
    }

    return undefined;
  }

  private async downloadBinary(
    storageDir: string,
    release: GithubRelease,
    version: string,
  ): Promise<string> {
    const assetInfo = resolveAssetName();
    const asset = release.assets.find(
      (item) => item.name === assetInfo.filename,
    );
    if (!asset) {
      throw new Error(
        `No compatible Codebook build (${assetInfo.descriptor}) was found in the ${version} release.`,
      );
    }

    const versionDir = path.join(storageDir, versionDirectoryName(version));
    await fsp.mkdir(versionDir, { recursive: true });

    const tempFile = await fsp.mkdtemp(
      path.join(os.tmpdir(), "codebook-download-"),
    );
    const archivePath = path.join(tempFile, assetInfo.filename);

    try {
      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: "Codebook",
        },
        async (progress) => {
          progress.report({ message: `Downloading ${asset.name}` });
          await downloadFile(
            asset.browser_download_url,
            archivePath,
            (percent) => {
              if (percent > 0) {
                progress.report({
                  message: `Downloading ${asset.name} (${Math.floor(percent * 100)}%)`,
                });
              }
            },
          );

          progress.report({ message: "Extracting archive" });
          if (assetInfo.type === "zip") {
            const zip = new AdmZip(archivePath);
            zip.extractAllTo(versionDir, true);
          } else {
            await tar.x({ cwd: versionDir, file: archivePath });
          }
        },
      );
    } finally {
      await cleanupTemp(tempFile);
    }

    const binaryPath = path.join(versionDir, BINARY_FILENAME);
    if (!(await fileExists(binaryPath))) {
      throw new Error(`Downloaded archive did not contain ${BINARY_FILENAME}`);
    }

    if (process.platform !== "win32") {
      await fsp.chmod(binaryPath, 0o755);
    }

    await fsp.writeFile(
      path.join(storageDir, VERSION_FILENAME),
      version,
      "utf8",
    );
    await this.cleanupOldVersions(storageDir, versionDirectoryName(version));

    this.logger.appendLine(
      `Installed codebook-lsp ${version} to ${binaryPath}`,
    );
    return binaryPath;
  }

  private async cleanupOldVersions(
    storageDir: string,
    keepDirName: string,
  ): Promise<void> {
    const entries = await fsp.readdir(storageDir, { withFileTypes: true });
    await Promise.all(
      entries.map(async (entry) => {
        if (
          entry.isDirectory() &&
          entry.name.startsWith("codebook-lsp-") &&
          entry.name !== keepDirName
        ) {
          await fsp.rm(path.join(storageDir, entry.name), {
            recursive: true,
            force: true,
          });
        }
      }),
    );
  }
}

function versionDirectoryName(version: string): string {
  return `codebook-lsp-${version}`;
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
  try {
    const result = await which(BINARY_BASENAME, { nothrow: true });
    return result ?? undefined;
  } catch {
    return undefined;
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

function resolveAssetName(): {
  filename: string;
  descriptor: string;
  type: ArchiveType;
} {
  let archPart: string;
  if (process.arch === "x64") {
    archPart = "x86_64";
  } else if (process.arch === "arm64") {
    archPart = "aarch64";
  } else {
    throw new Error(`Unsupported CPU architecture: ${process.arch}`);
  }

  let osPart: string;
  let extension: string;
  let type: ArchiveType;

  switch (process.platform) {
    case "darwin":
      osPart = "apple-darwin";
      extension = "tar.gz";
      type = "tar-gz";
      break;
    case "linux":
      osPart = "unknown-linux-musl";
      extension = "tar.gz";
      type = "tar-gz";
      break;
    case "win32":
      osPart = "pc-windows-msvc";
      extension = "zip";
      type = "zip";
      break;
    default:
      throw new Error(`Unsupported operating system: ${process.platform}`);
  }

  return {
    filename: `codebook-lsp-${archPart}-${osPart}.${extension}`,
    descriptor: `${archPart}-${osPart}`,
    type,
  };
}

function extractReleaseVersion(release: GithubRelease): string {
  return release.tag_name ?? release.name ?? String(release.id);
}

async function fetchDesiredRelease(
  includePrerelease: boolean,
): Promise<GithubRelease> {
  if (includePrerelease) {
    const releases = await requestJson<GithubRelease[]>(
      "https://api.github.com/repos/blopker/codebook/releases",
    );
    const release = releases.find((item) => item.assets?.length);
    if (!release) {
      throw new Error("No releases with downloadable assets were found.");
    }
    return release;
  }

  return requestJson<GithubRelease>(
    "https://api.github.com/repos/blopker/codebook/releases/latest",
  );
}

async function requestJson<T>(url: string): Promise<T> {
  const response = await httpRequest(url, {
    headers: {
      Accept: "application/vnd.github+json",
    },
  });

  const chunks: Buffer[] = [];
  for await (const chunk of response) {
    chunks.push(Buffer.from(chunk));
  }

  return JSON.parse(Buffer.concat(chunks).toString("utf8")) as T;
}

async function downloadFile(
  url: string,
  destination: string,
  progress?: (percent: number) => void,
): Promise<void> {
  const response = await httpRequest(url, {
    headers: {
      Accept: "application/octet-stream",
    },
  });

  const totalSize = Number(response.headers["content-length"]);
  let downloaded = 0;

  if (progress && Number.isFinite(totalSize)) {
    response.on("data", (chunk: Buffer) => {
      downloaded += chunk.length;
      if (totalSize > 0) {
        progress(downloaded / totalSize);
      }
    });
  }

  await pipeline(response, fs.createWriteStream(destination));
}

async function httpRequest(
  url: string,
  options: https.RequestOptions = {},
  redirectCount = 0,
): Promise<IncomingMessage> {
  if (redirectCount > MAX_REDIRECTS) {
    throw new Error("Too many redirects while downloading Codebook");
  }

  return new Promise<IncomingMessage>((resolve, reject) => {
    const request = https.get(
      url,
      {
        ...options,
        headers: {
          "User-Agent": USER_AGENT,
          ...options.headers,
        },
      },
      (response) => {
        const location = response.headers.location;
        if (
          response.statusCode &&
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          location
        ) {
          response.resume();
          const redirectUrl = new URL(location, url).toString();
          resolve(httpRequest(redirectUrl, options, redirectCount + 1));
          return;
        }

        if (
          response.statusCode &&
          response.statusCode >= 200 &&
          response.statusCode < 300
        ) {
          resolve(response);
          return;
        }

        response.resume();
        reject(
          new Error(
            `Request to ${url} failed with status ${response.statusCode}`,
          ),
        );
      },
    );

    request.setTimeout(30_000, () => {
      request.destroy(new Error("Request timed out while contacting GitHub"));
    });

    request.on("error", reject);
  });
}

async function cleanupTemp(dir: string): Promise<void> {
  await fsp.rm(dir, { recursive: true, force: true });
}
