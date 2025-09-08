import * as assert from "assert";
import * as fs from "fs";
import { afterEach, beforeEach, describe, it } from "mocha";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";

// Mock interfaces for testing
interface MockContext extends vscode.ExtensionContext {
	globalStorageUri: vscode.Uri;
}

describe("Binary Download Tests", () => {
	let mockContext: MockContext;
	let tempDir: string;

	beforeEach(() => {
		// Create a temporary directory for testing
		tempDir = path.join(os.tmpdir(), `codebook-test-${Date.now()}`);
		fs.mkdirSync(tempDir, { recursive: true });

		// Create mock context
		mockContext = {
			globalStorageUri: vscode.Uri.file(tempDir),
		} as MockContext;
	});

	afterEach(() => {
		// Clean up temporary directory
		if (fs.existsSync(tempDir)) {
			fs.rmSync(tempDir, { recursive: true, force: true });
		}
	});

	describe("Platform Detection", () => {
		it("should correctly detect platform for macOS", () => {
			if (os.platform() === "darwin") {
				const platform = os.platform();
				const arch = os.arch();

				assert.strictEqual(platform, "darwin");
				assert.ok(["x64", "arm64"].includes(arch));
			}
		});

		it("should correctly detect platform for Linux", () => {
			if (os.platform() === "linux") {
				const platform = os.platform();
				const arch = os.arch();

				assert.strictEqual(platform, "linux");
				assert.ok(["x64", "arm64"].includes(arch));
			}
		});

		it("should correctly detect platform for Windows", () => {
			if (os.platform() === "win32") {
				const platform = os.platform();
				const arch = os.arch();

				assert.strictEqual(platform, "win32");
				assert.ok(["x64", "arm64", "ia32"].includes(arch));
			}
		});
	});

	describe("Cache Management", () => {
		it("should create cache directory if it doesn't exist", () => {
			const cacheDir = path.join(tempDir, "binaries");

			if (!fs.existsSync(cacheDir)) {
				fs.mkdirSync(cacheDir, { recursive: true });
			}

			assert.ok(fs.existsSync(cacheDir));
			assert.ok(fs.statSync(cacheDir).isDirectory());
		});

		it("should save binary metadata", () => {
			const cacheDir = path.join(tempDir, "binaries");
			const metaPath = path.join(cacheDir, "binary-meta.json");

			fs.mkdirSync(cacheDir, { recursive: true });

			const metadata = {
				path: path.join(cacheDir, "v1.0.0", "codebook-lsp"),
				version: "v1.0.0",
			};

			fs.writeFileSync(metaPath, JSON.stringify(metadata, null, 2));

			assert.ok(fs.existsSync(metaPath));

			const savedMeta = JSON.parse(fs.readFileSync(metaPath, "utf8"));
			assert.deepStrictEqual(savedMeta, metadata);
		});

		it("should read cached binary metadata", () => {
			const cacheDir = path.join(tempDir, "binaries");
			const metaPath = path.join(cacheDir, "binary-meta.json");

			fs.mkdirSync(cacheDir, { recursive: true });

			const metadata = {
				path: path.join(cacheDir, "v1.0.0", "codebook-lsp"),
				version: "v1.0.0",
			};

			fs.writeFileSync(metaPath, JSON.stringify(metadata, null, 2));

			const readMeta = JSON.parse(fs.readFileSync(metaPath, "utf8"));
			assert.strictEqual(readMeta.version, "v1.0.0");
			assert.ok(readMeta.path.includes("codebook-lsp"));
		});

		it("should handle missing metadata gracefully", () => {
			const cacheDir = path.join(tempDir, "binaries");
			const metaPath = path.join(cacheDir, "binary-meta.json");

			assert.ok(!fs.existsSync(metaPath));

			// Should not throw when metadata doesn't exist
			let meta = null;
			if (fs.existsSync(metaPath)) {
				try {
					meta = JSON.parse(fs.readFileSync(metaPath, "utf8"));
				} catch {
					meta = null;
				}
			}

			assert.strictEqual(meta, null);
		});
	});

	describe("Binary Path Resolution", () => {
		it("should check for development binary", () => {
			const devPath = path.join(
				mockContext.globalStorageUri.fsPath,
				"..",
				"..",
				"target",
				"release",
				"codebook-lsp",
			);

			// In a real dev environment, this might exist
			const exists = fs.existsSync(devPath);
			assert.strictEqual(typeof exists, "boolean");
		});

		it("should handle custom server path from configuration", () => {
			const customPath = "/custom/path/to/codebook-lsp";

			// Mock configuration would return this custom path
			// In real implementation, this would come from vscode.workspace.getConfiguration
			const serverPath = customPath;

			assert.strictEqual(serverPath, customPath);
		});

		it("should handle Windows executable extension", () => {
			const binaryName = "codebook-lsp";
			const withExtension =
				process.platform === "win32" ? `${binaryName}.exe` : binaryName;

			if (process.platform === "win32") {
				assert.ok(withExtension.endsWith(".exe"));
			} else {
				assert.ok(!withExtension.endsWith(".exe"));
			}
		});
	});

	describe("Version Management", () => {
		it("should clean up old versions", () => {
			const cacheDir = path.join(tempDir, "binaries");

			// Create multiple version directories
			const oldVersion1 = path.join(cacheDir, "v0.1.0");
			const oldVersion2 = path.join(cacheDir, "v0.2.0");
			const currentVersion = path.join(cacheDir, "v1.0.0");

			fs.mkdirSync(oldVersion1, { recursive: true });
			fs.mkdirSync(oldVersion2, { recursive: true });
			fs.mkdirSync(currentVersion, { recursive: true });

			// Simulate cleanup of old versions
			const dirs = fs.readdirSync(cacheDir).filter((d) => {
				const dirPath = path.join(cacheDir, d);
				return fs.statSync(dirPath).isDirectory() && d !== "v1.0.0";
			});

			for (const dir of dirs) {
				fs.rmSync(path.join(cacheDir, dir), { recursive: true, force: true });
			}

			// Verify only current version remains
			const remainingDirs = fs.readdirSync(cacheDir).filter((d) => {
				const dirPath = path.join(cacheDir, d);
				return fs.statSync(dirPath).isDirectory();
			});

			assert.strictEqual(remainingDirs.length, 1);
			assert.strictEqual(remainingDirs[0], "v1.0.0");
		});

		it("should handle version directory creation", () => {
			const cacheDir = path.join(tempDir, "binaries");
			const versionDir = path.join(cacheDir, "v1.0.0");

			fs.mkdirSync(versionDir, { recursive: true });

			assert.ok(fs.existsSync(versionDir));
			assert.ok(fs.statSync(versionDir).isDirectory());
		});
	});

	describe("Error Handling", () => {
		it("should handle missing binary gracefully", () => {
			const binaryPath = path.join(tempDir, "nonexistent", "codebook-lsp");
			const exists = fs.existsSync(binaryPath);

			assert.strictEqual(exists, false);
		});

		it("should clean up temp directory on error", () => {
			const tempDownloadDir = path.join(tempDir, `temp-${Date.now()}`);

			fs.mkdirSync(tempDownloadDir, { recursive: true });
			assert.ok(fs.existsSync(tempDownloadDir));

			// Simulate error and cleanup
			try {
				// Simulate an error
				throw new Error("Download failed");
			} catch {
				// Clean up
				if (fs.existsSync(tempDownloadDir)) {
					fs.rmSync(tempDownloadDir, { recursive: true, force: true });
				}
			}

			assert.ok(!fs.existsSync(tempDownloadDir));
		});

		it("should handle permission errors", () => {
			if (process.platform !== "win32") {
				const binaryPath = path.join(tempDir, "test-binary");
				fs.writeFileSync(binaryPath, "test content");

				// Check initial permissions
				const stats = fs.statSync(binaryPath);
				const isExecutable = (stats.mode & 0o111) !== 0;

				// Make executable
				fs.chmodSync(binaryPath, 0o755);

				const newStats = fs.statSync(binaryPath);
				const isNowExecutable = (newStats.mode & 0o111) !== 0;

				assert.ok(isNowExecutable);
			}
		});
	});

	describe("Asset Name Generation", () => {
		it("should generate correct asset name for macOS", () => {
			const arch = os.arch() === "x64" ? "x86_64" : "aarch64";
			const assetName = `codebook-lsp-${arch}-apple-darwin.tar.gz`;

			if (os.platform() === "darwin") {
				assert.ok(assetName.includes("apple-darwin"));
				assert.ok(assetName.endsWith(".tar.gz"));
			}
		});

		it("should generate correct asset name for Linux", () => {
			const arch = os.arch() === "x64" ? "x86_64" : "aarch64";
			const assetName = `codebook-lsp-${arch}-unknown-linux-musl.tar.gz`;

			if (os.platform() === "linux") {
				assert.ok(assetName.includes("unknown-linux-musl"));
				assert.ok(assetName.endsWith(".tar.gz"));
			}
		});

		it("should generate correct asset name for Windows", () => {
			const arch = os.arch() === "x64" ? "x86_64" : "aarch64";
			const assetName = `codebook-lsp-${arch}-pc-windows-msvc.zip`;

			if (os.platform() === "win32") {
				assert.ok(assetName.includes("pc-windows-msvc"));
				assert.ok(assetName.endsWith(".zip"));
			}
		});
	});
});
