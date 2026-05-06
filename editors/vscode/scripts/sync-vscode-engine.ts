import fs from "node:fs";
import path from "node:path";
import { $ } from "bun";

const packagePath = path.join(import.meta.dir, "..", "package.json");
const monthsBack = Number.parseInt(process.argv[2] ?? "3", 10);

if (!Number.isInteger(monthsBack) || monthsBack < 1) {
  throw new Error("Usage: bun run sync-vscode-engine [months-back]");
}

type RegistryResponse = {
  time: Record<string, string>;
};

function compareVersions(a: string, b: string): number {
  const aParts = a.split(".").map((part) => Number.parseInt(part, 10));
  const bParts = b.split(".").map((part) => Number.parseInt(part, 10));

  for (let i = 0; i < Math.max(aParts.length, bParts.length); i += 1) {
    const difference = (aParts[i] ?? 0) - (bParts[i] ?? 0);
    if (difference !== 0) return difference;
  }

  return 0;
}

function cutoffDate(months: number): Date {
  const cutoff = new Date();
  cutoff.setMonth(cutoff.getMonth() - months);
  return cutoff;
}

const response = await fetch("https://registry.npmjs.org/@types%2Fvscode");
if (!response.ok) {
  throw new Error(`Failed to fetch @types/vscode metadata: ${response.status}`);
}

const registry = (await response.json()) as RegistryResponse;
const cutoff = cutoffDate(monthsBack);
const version = Object.entries(registry.time)
  .filter(([candidate]) => /^\d+\.\d+\.\d+$/.test(candidate))
  .filter(([, publishedAt]) => new Date(publishedAt) <= cutoff)
  .map(([candidate]) => candidate)
  .sort(compareVersions)
  .at(-1);

if (!version) {
  throw new Error(`No @types/vscode version found before ${cutoff.toISOString()}`);
}

const packageJson = JSON.parse(fs.readFileSync(packagePath, "utf-8"));
const previousEngine = packageJson.engines.vscode;
const previousTypes = packageJson.devDependencies["@types/vscode"];

packageJson.engines.vscode = `^${version}`;
packageJson.devDependencies["@types/vscode"] = version;

fs.writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);

await $`bun install`;

console.log(`Synced VS Code engine to ${version}`);
console.log(`engines.vscode: ${previousEngine} -> ^${version}`);
console.log(`@types/vscode: ${previousTypes} -> ${version}`);
