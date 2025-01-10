import { afterAll, afterEach, beforeAll, expect, test } from "bun:test";
import { LSPTestClient } from "./client";

// async function makeClient() {
//   const client = new LSPTestClient("../target/debug/codebook-lsp");
//   await client.start();
//   return client;
// }
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

test("should provide diagnostics for text", async (done) => {
  // const languageClient = await makeClient();
  languageClient.on("textDocument/publishDiagnostics", (params) => {
    console.log("Received diagnostics:", params);
    expect(params).toBeDefined();
    expect(false);
    done();
    // assert.ok(Array.isArray(completions));
  });
  languageClient.sendNotification("textDocument/didOpen", {
    textDocument: {
      uri: "file:///test.txt",
      languageId: "plaintext",
      version: 1,
      text: "Hello, Wolrd!",
    },
  });
});

test("should provide diagnostics for code", async (done) => {
  // const languageClient = await makeClient();
  languageClient.on("textDocument/publishDiagnostics", (params) => {
    console.log("Received diagnostics:", params);
    expect(params).toBeDefined();
    expect(false);
    done();
    // assert.ok(Array.isArray(completions));
  });
  languageClient.sendNotification("textDocument/didOpen", {
    textDocument: {
      uri: "file:///test.rs",
      languageId: "rust",
      version: 1,
      text: 'fn main() { println!("Hello, Wolrd!"); }',
    },
  });
});

test("should only highlight word in code", async (done) => {
  // const languageClient = await makeClient();
  languageClient.on("textDocument/publishDiagnostics", (params) => {
    console.log("Received diagnostics:", params);
    expect(params).toBeDefined();
    expect(false);
    done();
    // assert.ok(Array.isArray(completions));
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
});
