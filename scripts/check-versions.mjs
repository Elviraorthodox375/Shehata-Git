// Copyright (c) 2026 Dr Mohamed Shehata. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root.

// Four files carry the product version, and every release bumps all four by
// hand. When one is missed the installer, the CLI, and the release tag start
// describing different builds — so this fails the build instead.
//
// Run with a tag to also check it matches, e.g.
//   node scripts/check-versions.mjs v0.1.21

import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

/** Read the workspace version from Cargo.toml without a TOML parser. */
function cargoWorkspaceVersion(relativePath) {
  const text = readFileSync(join(repoRoot, relativePath), "utf8");
  const section = text.split(/^\[/m).find((part) => part.startsWith("workspace.package]"));
  if (!section) throw new Error(`${relativePath}: no [workspace.package] section`);
  const match = section.match(/^\s*version\s*=\s*"([^"]+)"/m);
  if (!match) throw new Error(`${relativePath}: no version under [workspace.package]`);
  return match[1];
}

function jsonVersion(relativePath) {
  const parsed = JSON.parse(readFileSync(join(repoRoot, relativePath), "utf8"));
  if (!parsed.version) throw new Error(`${relativePath}: no version field`);
  return parsed.version;
}

const sources = [
  ["Cargo.toml", cargoWorkspaceVersion("Cargo.toml")],
  ["package.json", jsonVersion("package.json")],
  ["apps/desktop/package.json", jsonVersion("apps/desktop/package.json")],
  [
    "apps/desktop/src-tauri/tauri.conf.json",
    jsonVersion("apps/desktop/src-tauri/tauri.conf.json"),
  ],
];

const distinct = [...new Set(sources.map(([, version]) => version))];
const problems = [];

if (distinct.length !== 1) {
  problems.push("Version mismatch across manifests:");
  for (const [file, version] of sources) problems.push(`  ${version}  ${file}`);
}

// A tag that disagrees with the manifests would publish an installer whose
// file name and contents describe different releases.
const tag = process.argv[2];
if (tag) {
  const expected = `v${sources[0][1]}`;
  if (tag !== expected) {
    problems.push(`Tag ${tag} does not match the manifest version (${expected}).`);
  }
}

if (problems.length > 0) {
  console.error(problems.join("\n"));
  process.exit(1);
}

console.log(`Version ${distinct[0]} is consistent across ${sources.length} manifests.`);
