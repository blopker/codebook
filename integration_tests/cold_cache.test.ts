import { afterAll, beforeAll, expect, test } from "bun:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { LSPTestClient } from "./client";
import { waitForDiagnostics } from "./utils";

// Exercises the background dictionary prefetch end-to-end: a server with an
// empty cache must respond to didOpen without blocking on downloads (an
// early, possibly empty, diagnostics publish), then re-check the still-open
// document on its own once the dictionaries land.

let client: LSPTestClient;
let isolatedHome: string;

beforeAll(async () => {
  // A fresh HOME/XDG keeps the dictionary cache cold and the user's real
  // global config out of the test.
  isolatedHome = fs.mkdtempSync(path.join(os.tmpdir(), "codebook-cold-"));
  client = new LSPTestClient("../target/release/codebook-lsp", {
    HOME: isolatedHome,
    XDG_DATA_HOME: path.join(isolatedHome, "data"),
    XDG_CONFIG_HOME: path.join(isolatedHome, "config"),
  });
  await client.start();
});

afterAll(async () => {
  if (client) {
    await client.stop();
  }
  fs.rmSync(isolatedHome, { recursive: true, force: true });
});

test(
  "cold cache: didOpen answers promptly, diagnostics arrive after prefetch",
  async () => {
    const uri = "file:///cold.txt";
    const firstPublish = waitForDiagnostics(client, { uri, timeoutMs: 5000 });
    const nonEmptyPublish = waitForDiagnostics(client, {
      uri,
      nonEmpty: true,
      timeoutMs: 30000,
    });

    client.sendNotification("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "plaintext",
        version: 1,
        text: "Hello, Wolrd!",
      },
    });

    // The check must not block on the download: some publish (empty is fine)
    // arrives well before a cold download could complete on a slow link.
    await firstPublish;

    // Without any further notifications, the prefetch worker downloads the
    // dictionaries and the server re-checks the open document on its own.
    const params = await nonEmptyPublish;
    expect(params.diagnostics.length).toBeGreaterThan(0);
    expect(
      params.diagnostics.some((d) => d.message.includes("Wolrd")),
    ).toBeTrue();
  },
  40000,
);
