// bun ts script to release all codebook crates.
// This script will:
// - get version from codebook-lsp/Cargo.toml
// - ask what to change the version to
// - update the version in all crates' Cargo.toml files
// - commit the change
// - tag the commit
// - push the commit and tag

import fs, { globSync } from "node:fs";
import path from "node:path";
import { $ } from "bun";

function bumpVersionMinor(version: string) {
  const parts = version.split(".").map((d) => Number.parseInt(d));
  parts[parts.length - 1] += 1;
  return parts.join(".");
}

await $`git checkout main`;
await $`git pull origin main`;
// error if we're dirty
try {
  await $`git diff --exit-code > /dev/null`;
} catch (_) {
  console.error("Warning: git is dirty");
  const answer = prompt("Continue? [Y/n]") ?? "Y";
  if (answer.toLowerCase() !== "y") {
    process.exit(1);
  }
}

// Get reference version from codebook-lsp
const lspPath = path.join(__dirname, "..", "crates", "codebook-lsp");
const lspCargoPath = path.join(lspPath, "Cargo.toml");
const lspCargo = fs.readFileSync(lspCargoPath, "utf-8");
const version = lspCargo.match(/version = "(.*)"/)?.[1] ?? "None";
console.log("Current version:", version);
const maybeNewVersion = bumpVersionMinor(version);
const newVersion =
  prompt(`What is the new version? [${maybeNewVersion}]`) ?? maybeNewVersion;
console.log("New version:", newVersion);

// Find all Cargo.toml files in crates directory
const tomlPaths = globSync(
  path.join(__dirname, "..", "crates", "**", "Cargo.toml"),
);

console.log("Updating version in crates:", tomlPaths.join(", "));

// Update version in all crates
for (const toml of tomlPaths) {
  const cargo = fs.readFileSync(toml, "utf-8");
  const cargoVersion = cargo.match(/version = "([^"]*)"/)?.[1];
  if (!cargoVersion) continue;

  const newCargo = cargo.replace(
    /version = "[^"]*"/,
    `version = "${newVersion}"`,
  );
  fs.writeFileSync(toml, newCargo);
  console.log(`Updated ${toml} from ${cargoVersion} to ${newVersion}`);
}

// Point the workspace's dependencies on its own crates at the new version.
// Without this, a minor bump (0.3.x -> 0.4.0) leaves a `version = "0.3.0"`
// requirement that the local crates no longer satisfy.
const workspaceCargoPath = path.join(__dirname, "..", "Cargo.toml");
const workspaceCargo = fs.readFileSync(workspaceCargoPath, "utf-8");
const newWorkspaceCargo = workspaceCargo.replace(
  /^(codebook\w* = \{ path = "crates\/[^"]+", version = ")[^"]*(" \})$/gm,
  `$1${newVersion}$2`,
);
fs.writeFileSync(workspaceCargoPath, newWorkspaceCargo);
console.log(`Updated workspace dependencies to ${newVersion}`);

// sleep to let the user see the changes
await new Promise((resolve) => setTimeout(resolve, 500));
await $`cargo update --workspace`; // update the lock file
await $`git add ."`;
await $`git commit -m "release codebook ${newVersion}"`;
await $`git tag "v${newVersion}"`;
await $`git push origin HEAD --tags`;

console.log("Released codebook", newVersion);
console.log("Go to https://github.com/blopker/codebook/releases to publish.");
