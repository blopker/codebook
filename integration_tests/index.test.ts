import { afterAll, afterEach, beforeAll, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { LSPTestClient } from "./client";
import { getLanguageFromFileName } from "./utils";

let languageClient: LSPTestClient;

beforeAll(async () => {
  // Create client
  languageClient = new LSPTestClient("../target/debug/codebook-lsp");

  // Start client
  await languageClient.start();
});

afterAll(async () => {
  if (languageClient) {
    await languageClient.stop();
  }
});

afterEach(async () => {
  languageClient.removeAllListeners();
});

test("should provide diagnostics for text", async () => {
  await new Promise<void>((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error("Timeout waiting for diagnostics for text"));
    }, 5000);

    try {
      languageClient.once("textDocument/publishDiagnostics", (params) => {
        try {
          console.log("Received diagnostics:", params);
          expect(params).toBeDefined();
          expect(params.diagnostics.length).toBeGreaterThan(0);
          clearTimeout(timeoutId);
          resolve();
        } catch (error) {
          clearTimeout(timeoutId);
          reject(error);
        }
      });

      languageClient.sendNotification("textDocument/didOpen", {
        textDocument: {
          uri: "file:///test.txt",
          languageId: "plaintext",
          version: 1,
          text: "Hello, Wolrd!",
        },
      });
    } catch (error) {
      clearTimeout(timeoutId);
      reject(error);
    }
  });
});

test("should provide diagnostics for code", async () => {
  await new Promise<void>((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error("Timeout waiting for diagnostics for code"));
    }, 5000);

    try {
      languageClient.once("textDocument/publishDiagnostics", (params) => {
        try {
          console.log("Received diagnostics:", params);
          expect(params).toBeDefined();
          expect(params.diagnostics.length).toBeGreaterThan(0);
          clearTimeout(timeoutId);
          resolve();
        } catch (error) {
          clearTimeout(timeoutId);
          reject(error);
        }
      });

      languageClient.sendNotification("textDocument/didOpen", {
        textDocument: {
          uri: "file:///test.rs",
          languageId: "rust",
          version: 1,
          text: 'fn main() { println!("Hello, Wolrd!"); }',
        },
      });
    } catch (error) {
      clearTimeout(timeoutId);
      reject(error);
    }
  });
});

test("should only highlight word in code", async () => {
  await new Promise<void>((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error("Timeout waiting for diagnostics for python"));
    }, 5000);

    try {
      languageClient.once("textDocument/publishDiagnostics", (params) => {
        try {
          // console.log("Received diagnostics:", params);
          expect(params).toBeDefined();
          expect(params.diagnostics.length).toBeGreaterThan(0);
          clearTimeout(timeoutId);
          resolve();
        } catch (error) {
          clearTimeout(timeoutId);
          reject(error);
        }
      });

      languageClient.sendNotification("textDocument/didOpen", {
        textDocument: {
          uri: "file:///example.py",
          languageId: "python",
          version: 1,
          text: `# Example Pthon fie
          def main():
              print("Hello, Wolrd!")

          if __name__ == "__main__":
              main()
          `,
        },
      });
    } catch (error) {
      clearTimeout(timeoutId);
      reject(error);
    }
  });
});

test("should provide diagnostics for all example files", async () => {
  const exampleDir = path.join(__dirname, "../examples");
  const files = fs.readdirSync(exampleDir);

  for (const file of files) {
    const filePath = path.join(exampleDir, file);
    const content = fs.readFileSync(filePath, { encoding: "utf8" });
    console.log(content);

    await new Promise<void>((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(new Error(`Timeout waiting for diagnostics for ${file}`));
      }, 5000);

      try {
        languageClient.once("textDocument/publishDiagnostics", (params) => {
          try {
            console.log(`Received diagnostics for ${file}:`, params);
            expect(params).toBeDefined();
            expect(params.diagnostics.length).toBeGreaterThan(0);
            clearTimeout(timeoutId);
            resolve();
          } catch (error) {
            clearTimeout(timeoutId);
            reject(error);
          }
        });
        console.log(`Sending didOpen notification for ${file}`);
        languageClient.sendNotification("textDocument/didOpen", {
          textDocument: {
            uri: `file:///${file}`,
            languageId: getLanguageFromFileName(file),
            version: 1,
            text: content,
          },
        });
      } catch (error) {
        clearTimeout(timeoutId);
        reject(error);
      }
    });
  }
});
