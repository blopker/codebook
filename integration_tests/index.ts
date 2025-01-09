import { LSPTestClient } from "./client";
// Example usage
async function main() {
  const client = new LSPTestClient("../target/debug/codebook-lsp");

  try {
    await client.start();

    // Listen for diagnostics
    client.on("textDocument/publishDiagnostics", (params) => {
      console.log("Received diagnostics:", params);
    });
    const messagePromise = client.waitForNextMessage();
    // Example: Send a document
    await client.sendNotification("textDocument/didOpen", {
      textDocument: {
        uri: "file:///test.txt",
        languageId: "plaintext",
        version: 1,
        text: "Hello, Wolrd!",
      },
    });

    await messagePromise;
    await client.stop();
  } catch (error) {
    console.error("Error:", error);
  }
}

main();
