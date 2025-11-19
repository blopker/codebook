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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.CodebookBinaryManager = void 0;
const vscode = __importStar(require("vscode"));
const fs = __importStar(require("node:fs"));
const fsp = __importStar(require("node:fs/promises"));
const path = __importStar(require("node:path"));
const os = __importStar(require("node:os"));
const https = __importStar(require("node:https"));
const promises_1 = require("node:stream/promises");
const adm_zip_1 = __importDefault(require("adm-zip"));
const tar_1 = __importDefault(require("tar"));
const which_1 = __importDefault(require("which"));
const BINARY_BASENAME = "codebook-lsp";
const BINARY_FILENAME = process.platform === "win32" ? `${BINARY_BASENAME}.exe` : BINARY_BASENAME;
const VERSION_FILENAME = "codebook.version";
const USER_AGENT = "codebook-vscode-extension";
const MAX_REDIRECTS = 5;
class CodebookBinaryManager {
    context;
    logger;
    binaryPath;
    constructor(context, logger) {
        this.context = context;
        this.logger = logger;
    }
    async getBinaryPath(forceRedownload = false) {
        if (this.binaryPath && !forceRedownload) {
            return this.binaryPath;
        }
        const config = vscode.workspace.getConfiguration("codebook");
        const explicitPath = config.get("binaryPath")?.trim();
        if (explicitPath) {
            await ensureExecutable(explicitPath);
            this.logger.appendLine(`Using codebook binary from configuration: ${explicitPath}`);
            this.binaryPath = explicitPath;
            return explicitPath;
        }
        if (!forceRedownload) {
            const systemBinary = await findOnPath();
            if (systemBinary) {
                this.logger.appendLine(`Using codebook binary from PATH: ${systemBinary}`);
                this.binaryPath = systemBinary;
                return systemBinary;
            }
        }
        const storageDir = await this.ensureStorageDir();
        const cached = await this.getCachedBinary(storageDir);
        const enablePrerelease = config.get("enablePrerelease", false);
        try {
            const release = await fetchDesiredRelease(enablePrerelease);
            const releaseVersion = extractReleaseVersion(release);
            if (!forceRedownload &&
                cached &&
                cached.version === releaseVersion &&
                (await fileExists(cached.path))) {
                this.logger.appendLine(`Using cached codebook binary version ${releaseVersion}`);
                this.binaryPath = cached.path;
                return cached.path;
            }
            const binaryPath = await this.downloadBinary(storageDir, release, releaseVersion);
            this.binaryPath = binaryPath;
            return binaryPath;
        }
        catch (error) {
            if (cached && (await fileExists(cached.path))) {
                this.logger.appendLine(`Failed to update Codebook (${error}). Falling back to cached binary at ${cached.path}`);
                this.binaryPath = cached.path;
                return cached.path;
            }
            throw error;
        }
    }
    async invalidateCache() {
        this.binaryPath = undefined;
    }
    async ensureStorageDir() {
        const storageDir = this.context.globalStorageUri.fsPath;
        await fsp.mkdir(storageDir, { recursive: true });
        return storageDir;
    }
    async getCachedBinary(storageDir) {
        try {
            const versionFile = path.join(storageDir, VERSION_FILENAME);
            const version = (await fsp.readFile(versionFile, "utf8")).trim();
            if (!version) {
                return undefined;
            }
            const binaryPath = path.join(storageDir, versionDirectoryName(version), BINARY_FILENAME);
            if (await fileExists(binaryPath)) {
                return { path: binaryPath, version };
            }
        }
        catch {
            // ignore missing cache
        }
        return undefined;
    }
    async downloadBinary(storageDir, release, version) {
        const assetInfo = resolveAssetName();
        const asset = release.assets.find((item) => item.name === assetInfo.filename);
        if (!asset) {
            throw new Error(`No compatible Codebook build (${assetInfo.descriptor}) was found in the ${version} release.`);
        }
        const versionDir = path.join(storageDir, versionDirectoryName(version));
        await fsp.mkdir(versionDir, { recursive: true });
        const tempFile = await fsp.mkdtemp(path.join(os.tmpdir(), "codebook-download-"));
        const archivePath = path.join(tempFile, assetInfo.filename);
        try {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: "Codebook",
            }, async (progress) => {
                progress.report({ message: `Downloading ${asset.name}` });
                await downloadFile(asset.browser_download_url, archivePath, (percent) => {
                    if (percent > 0) {
                        progress.report({
                            message: `Downloading ${asset.name} (${Math.floor(percent * 100)}%)`,
                        });
                    }
                });
                progress.report({ message: "Extracting archive" });
                if (assetInfo.type === "zip") {
                    const zip = new adm_zip_1.default(archivePath);
                    zip.extractAllTo(versionDir, true);
                }
                else {
                    await tar_1.default.x({ cwd: versionDir, file: archivePath });
                }
            });
        }
        finally {
            await cleanupTemp(tempFile);
        }
        const binaryPath = path.join(versionDir, BINARY_FILENAME);
        if (!(await fileExists(binaryPath))) {
            throw new Error(`Downloaded archive did not contain ${BINARY_FILENAME}`);
        }
        if (process.platform !== "win32") {
            await fsp.chmod(binaryPath, 0o755);
        }
        await fsp.writeFile(path.join(storageDir, VERSION_FILENAME), version, "utf8");
        await this.cleanupOldVersions(storageDir, versionDirectoryName(version));
        this.logger.appendLine(`Installed codebook-lsp ${version} to ${binaryPath}`);
        return binaryPath;
    }
    async cleanupOldVersions(storageDir, keepDirName) {
        const entries = await fsp.readdir(storageDir, { withFileTypes: true });
        await Promise.all(entries.map(async (entry) => {
            if (entry.isDirectory() &&
                entry.name.startsWith("codebook-lsp-") &&
                entry.name !== keepDirName) {
                await fsp.rm(path.join(storageDir, entry.name), {
                    recursive: true,
                    force: true,
                });
            }
        }));
    }
}
exports.CodebookBinaryManager = CodebookBinaryManager;
function versionDirectoryName(version) {
    return `codebook-lsp-${version}`;
}
async function ensureExecutable(filePath) {
    try {
        await fsp.access(filePath, fs.constants.X_OK);
    }
    catch {
        throw new Error(`The configured codebook binary is not executable: ${filePath}`);
    }
}
async function findOnPath() {
    try {
        const result = await (0, which_1.default)(BINARY_BASENAME, { nothrow: true });
        return result ?? undefined;
    }
    catch {
        return undefined;
    }
}
async function fileExists(filePath) {
    try {
        await fsp.access(filePath, fs.constants.F_OK);
        return true;
    }
    catch {
        return false;
    }
}
function resolveAssetName() {
    let archPart;
    if (process.arch === "x64") {
        archPart = "x86_64";
    }
    else if (process.arch === "arm64") {
        archPart = "aarch64";
    }
    else {
        throw new Error(`Unsupported CPU architecture: ${process.arch}`);
    }
    let osPart;
    let extension;
    let type;
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
function extractReleaseVersion(release) {
    return release.tag_name ?? release.name ?? String(release.id);
}
async function fetchDesiredRelease(includePrerelease) {
    if (includePrerelease) {
        const releases = await requestJson("https://api.github.com/repos/blopker/codebook/releases");
        const release = releases.find((item) => item.assets?.length);
        if (!release) {
            throw new Error("No releases with downloadable assets were found.");
        }
        return release;
    }
    return requestJson("https://api.github.com/repos/blopker/codebook/releases/latest");
}
async function requestJson(url) {
    const response = await httpRequest(url, {
        headers: {
            Accept: "application/vnd.github+json",
        },
    });
    const chunks = [];
    for await (const chunk of response) {
        chunks.push(Buffer.from(chunk));
    }
    return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}
async function downloadFile(url, destination, progress) {
    const response = await httpRequest(url, {
        headers: {
            Accept: "application/octet-stream",
        },
    });
    const totalSize = Number(response.headers["content-length"]);
    let downloaded = 0;
    if (progress && Number.isFinite(totalSize)) {
        response.on("data", (chunk) => {
            downloaded += chunk.length;
            if (totalSize > 0) {
                progress(downloaded / totalSize);
            }
        });
    }
    await (0, promises_1.pipeline)(response, fs.createWriteStream(destination));
}
async function httpRequest(url, options = {}, redirectCount = 0) {
    if (redirectCount > MAX_REDIRECTS) {
        throw new Error("Too many redirects while downloading Codebook");
    }
    return new Promise((resolve, reject) => {
        const request = https.get(url, {
            ...options,
            headers: {
                "User-Agent": USER_AGENT,
                ...options.headers,
            },
        }, (response) => {
            const location = response.headers.location;
            if (response.statusCode &&
                response.statusCode >= 300 &&
                response.statusCode < 400 &&
                location) {
                response.resume();
                const redirectUrl = new URL(location, url).toString();
                resolve(httpRequest(redirectUrl, options, redirectCount + 1));
                return;
            }
            if (response.statusCode &&
                response.statusCode >= 200 &&
                response.statusCode < 300) {
                resolve(response);
                return;
            }
            response.resume();
            reject(new Error(`Request to ${url} failed with status ${response.statusCode}`));
        });
        request.setTimeout(30_000, () => {
            request.destroy(new Error("Request timed out while contacting GitHub"));
        });
        request.on("error", reject);
    });
}
async function cleanupTemp(dir) {
    await fsp.rm(dir, { recursive: true, force: true });
}
//# sourceMappingURL=binary.js.map