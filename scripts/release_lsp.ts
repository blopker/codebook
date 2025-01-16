// bun ts script to release the codebook-lsp crate.
// This script will:
// - get version from codebook-lsp/Cargo.toml
// - ask what to change the version to
// - update the version in codebook-lsp/Cargo.toml
// - commit the change
// - tag the commit
// - push the commit and tag

import fs from "node:fs";
import path from "node:path";
import { $ } from "bun";

function bumpVersionMinor(version: string) {
  const parts = version.split(".").map((d) => Number.parseInt(d));
  parts[parts.length - 1] += 1;
  return parts.join(".");
}

await $`git checkout main`;
// error if we're dirty
try {
  await $`git diff --exit-code > /dev/null`;
} catch (e) {
  console.error("Warning: git is dirty");
  const answer = prompt("Continue? [Y/n]") ?? "Y";
  if (answer.toLowerCase() !== "y") {
    process.exit(1);
  }
}

const lspPath = path.join(__dirname, "..", "codebook-lsp");
const cargoPath = path.join(lspPath, "Cargo.toml");
const cargo = fs.readFileSync(cargoPath, "utf-8");
const version = cargo.match(/version = "(.*)"/)?.[1] ?? "None";
console.log("Current version:", version);
const maybeNewVersion = bumpVersionMinor(version);
const newVersion =
  prompt(`What is the new version? [${maybeNewVersion}]`) ?? maybeNewVersion;
console.log("New version:", newVersion);
const newCargo = cargo.replace(version, newVersion);
fs.writeFileSync(cargoPath, newCargo);
// sleep to let the user see the change
await new Promise((resolve) => setTimeout(resolve, 500));
await $`cargo update --workspace`; // update the lock file
await $`git add ."`;
await $`git commit -m "release codebook-lsp ${newVersion}"`;
await $`git tag ${newVersion}`;
await $`git push origin HEAD --tags`;

console.log("Released codebook-lsp", newVersion);
console.log("Go to https://github.com/blopker/codebook/releases to publish.");
