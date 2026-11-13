// bun ts script to release all codebook crates.
// This script will:
// - get version from the root Cargo.toml [workspace.package]
// - ask what to change the version to
// - update the version in the root Cargo.toml
// - commit the change
// - tag the commit
// - push the commit and tag

import fs from "node:fs";
import path from "node:path";
import { $ } from "bun";

function bumpVersionPatch(version: string) {
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

// All crates inherit `version` from `[workspace.package]` in the root
// Cargo.toml, so that is the only version to read and bump.
const workspaceCargoPath = path.join(__dirname, "..", "Cargo.toml");
const workspaceCargo = fs.readFileSync(workspaceCargoPath, "utf-8");
const workspaceVersionRe = /^(\[workspace\.package\]\nversion = ")([^"]*)(")$/m;
const version = workspaceCargo.match(workspaceVersionRe)?.[2];
if (!version) {
  console.error(
    'Could not find `version = "..."` as the first line of [workspace.package] in Cargo.toml',
  );
  process.exit(1);
}
console.log("Current version:", version);
const maybeNewVersion = bumpVersionPatch(version);
const newVersion =
  prompt(`What is the new version? [${maybeNewVersion}]`) ?? maybeNewVersion;
console.log("New version:", newVersion);

// Also point the workspace's dependencies on its own crates at the new
// version. crates.io requires a version on path dependencies, and without
// this a minor bump (0.3.x -> 0.4.0) leaves a `version = "0.3.0"`
// requirement that the local crates no longer satisfy.
const newWorkspaceCargo = workspaceCargo
  .replace(workspaceVersionRe, `$1${newVersion}$3`)
  .replace(
    /^(codebook\w* = \{ path = "crates\/[^"]+", version = ")[^"]*(" \})$/gm,
    `$1${newVersion}$2`,
  );
fs.writeFileSync(workspaceCargoPath, newWorkspaceCargo);
console.log(`Updated workspace version and dependencies to ${newVersion}`);

// sleep to let the user see the changes
await new Promise((resolve) => setTimeout(resolve, 500));
await $`cargo update --workspace`; // update the lock file
await $`git add .`;
await $`git commit -m "release codebook ${newVersion}"`;
await $`git tag "v${newVersion}"`;
await $`git push origin HEAD --tags`;

console.log("Released codebook", newVersion);
console.log("Go to https://github.com/blopker/codebook/releases to publish.");
