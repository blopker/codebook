import { afterAll, afterEach, beforeAll, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { LSPTestClient } from "./client";
import { getLanguageFromFileName, waitForDiagnostics } from "./utils";

// Generous per-test budget: on a cold cache the server downloads
// dictionaries in the background and re-publishes when they land.
const TEST_TIMEOUT = 35000;

let languageClient: LSPTestClient;

beforeAll(async () => {
  // Create client
  languageClient = new LSPTestClient("../target/release/codebook-lsp");

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

test(
  "should provide diagnostics for text",
  async () => {
    const diagnostics = waitForDiagnostics(languageClient, {
      uri: "file:///test.txt",
      nonEmpty: true,
    });

    languageClient.sendNotification("textDocument/didOpen", {
      textDocument: {
        uri: "file:///test.txt",
        languageId: "plaintext",
        version: 1,
        text: "Hello, Wolrd!",
      },
    });

    const params = await diagnostics;
    console.log("Received diagnostics:", params);
    expect(params.diagnostics.length).toBeGreaterThan(0);
  },
  TEST_TIMEOUT,
);

test(
  "should provide diagnostics for code",
  async () => {
    const diagnostics = waitForDiagnostics(languageClient, {
      uri: "file:///test.rs",
      nonEmpty: true,
    });

    languageClient.sendNotification("textDocument/didOpen", {
      textDocument: {
        uri: "file:///test.rs",
        languageId: "rust",
        version: 1,
        text: 'fn main() { println!("Hello, Wolrd!"); }',
      },
    });

    const params = await diagnostics;
    console.log("Received diagnostics:", params);
    expect(params.diagnostics.length).toBeGreaterThan(0);
  },
  TEST_TIMEOUT,
);

test(
  "should only highlight word in code",
  async () => {
    const diagnostics = waitForDiagnostics(languageClient, {
      uri: "file:///example.py",
      nonEmpty: true,
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

    const params = await diagnostics;
    expect(params.diagnostics.length).toBeGreaterThan(0);
  },
  TEST_TIMEOUT,
);

test(
  "should provide diagnostics for all example files",
  async () => {
    const exampleDir = path.join(__dirname, "../examples");
    const files = fs.readdirSync(exampleDir);

    for (const file of files) {
      const filePath = path.join(exampleDir, file);
      const content = fs.readFileSync(filePath, { encoding: "utf8" });

      const diagnostics = waitForDiagnostics(languageClient, {
        uri: `file:///${file}`,
        nonEmpty: true,
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

      const params = await diagnostics;
      console.log(`Received diagnostics for ${file}:`, params);
      expect(params.diagnostics.length).toBeGreaterThan(0);
    }
  },
  TEST_TIMEOUT * 2,
);
