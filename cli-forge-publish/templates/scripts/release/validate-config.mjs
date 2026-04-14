#!/usr/bin/env node
// Shared config validation for the release pipeline. Checks required fields,
// scope consistency, sourceRepository match against GITHUB_REPOSITORY (CI),
// and npm/main/package.json placeholder status. Exits 0 on pass, 1 on fail.
//
// Called from release.yml "Verify release config" step and from
// sync-platform-packages.mjs so both locations use identical logic.

import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);

const config = JSON.parse(
  readFileSync(path.join(rootDir, "release/config.json"), "utf8"),
);

// --- Required fields ---
for (const field of ["cliName", "packageName", "sourceRepository"]) {
  const value = config[field];
  if (!value || /REPLACE_WITH_/.test(String(value))) {
    console.error(
      `release/config.json#${field} must be set (found ${JSON.stringify(value)}).`,
    );
    process.exit(1);
  }
}

// --- Scope consistency ---
const declaredScope = config.npmScope ?? null;
const nameStartsWithScope = config.packageName.startsWith("@");

if (nameStartsWithScope) {
  const inferredScope = config.packageName.slice(1).split("/")[0];
  if (declaredScope == null) {
    console.error(
      `release/config.json#packageName is scoped (${config.packageName}) but npmScope is null.`,
    );
    process.exit(1);
  }
  if (declaredScope !== inferredScope) {
    console.error(
      `release/config.json#npmScope (${declaredScope}) does not match packageName scope (${inferredScope}).`,
    );
    process.exit(1);
  }
} else if (declaredScope != null) {
  console.error(
    `release/config.json#npmScope is ${JSON.stringify(declaredScope)} but packageName is unscoped (${config.packageName}).`,
  );
  process.exit(1);
}

// --- sourceRepository matches GITHUB_REPOSITORY (CI only) ---
const ghRepo = process.env.GITHUB_REPOSITORY ?? "";
if (ghRepo && config.sourceRepository !== ghRepo) {
  console.error(
    `release/config.json#sourceRepository (${config.sourceRepository}) does not match GITHUB_REPOSITORY (${ghRepo}).`,
  );
  process.exit(1);
}

// --- Main package.json placeholders ---
const mainPkgPath = path.join(rootDir, "npm/main/package.json");
if (existsSync(mainPkgPath)) {
  const pkg = JSON.parse(readFileSync(mainPkgPath, "utf8"));
  for (const field of ["name", "description", "bin"]) {
    const val =
      typeof pkg[field] === "object"
        ? JSON.stringify(pkg[field])
        : String(pkg[field]);
    if (/REPLACE_WITH_/.test(val)) {
      console.error(`npm/main/package.json#${field} still has placeholders.`);
      process.exit(1);
    }
  }
  console.log(`Main package OK: ${pkg.name}`);
}

console.log(
  `Config OK: ${config.cliName} ${config.packageName} ${config.sourceRepository}`,
);
