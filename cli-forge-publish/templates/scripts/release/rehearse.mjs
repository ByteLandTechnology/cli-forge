#!/usr/bin/env node
// Rehearses the full release pipeline locally without pushing tags, publishing
// to npm, or creating a GitHub Release. Builds the binaries for every
// configured target, syncs the platform packages, then runs `npm publish
// --dry-run` for each one. Exits 0 when every step succeeds.
//
// On success or failure, all generated artifacts and all mutated files are
// restored so the working tree is identical to the pre-rehearsal state.
// This includes dist/, npm/platforms/, npm/main/package.json,
// npm/main/README.md. Build outputs go to an isolated temporary directory
// (CARGO_TARGET_DIR) so the real target/ is never touched.
//
// Prerequisites: cargo, rustup, and the cross-build toolchain must already be
// installed locally (zig + cargo-zigbuild for Linux targets, llvm-mingw for
// Windows targets). CI gets these from .github/actions/setup-build-env.
//
// Usage: node scripts/release/rehearse.mjs

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
  mkdtempSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);
const config = JSON.parse(
  readFileSync(path.join(rootDir, "release/config.json"), "utf8"),
);
const distDir = path.join(rootDir, "dist");
const platformsDir = path.join(rootDir, "npm/platforms");
const mainPkgPath = path.join(rootDir, "npm/main/package.json");
const mainReadmePath = path.join(rootDir, "npm/main/README.md");
const isolatedTargetDir = mkdtempSync(
  path.join(tmpdir(), "cli-forge-rehearse-"),
);
const cliName = config.cliName;
const pkgName = config.packageName;
const rehearsalVersion = "0.0.0-rehearsal";

// Delegate all field/scope validation to the shared script so config errors
// (including scope mismatches) are caught before the expensive build step.
const validateResult = spawnSync(
  process.execPath,
  [path.join(rootDir, "scripts/release/validate-config.mjs")],
  { cwd: rootDir, stdio: "inherit" },
);
if (validateResult.status !== 0) {
  throw new Error("Config validation failed.");
}

// --- Preflight: verify required tooling ------------------------------------
console.log("\n=== Preflight ===\n");

const hasLinux = config.targets.some((t) => t.rustTarget.includes("linux"));
const hasWindows = config.targets.some((t) => t.rustTarget.includes("windows"));

if (
  spawnSync("cargo", ["--version"], { encoding: "utf8", shell: true })
    .status !== 0
) {
  throw new Error("cargo not found. Install Rust: https://rustup.rs");
}
console.log("  cargo: OK");

if (hasLinux) {
  const zigcheck = spawnSync("cargo", ["zigbuild", "--version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    shell: true,
  });
  if (zigcheck.status !== 0) {
    throw new Error(
      "cargo-zigbuild not found. Install: cargo install cargo-zigbuild && brew install zig (macOS)",
    );
  }
  console.log("  cargo-zigbuild: OK");
}

const installedTargets =
  spawnSync("rustup", ["target", "list", "--installed"], {
    encoding: "utf8",
    shell: true,
  }).stdout ?? "";

if (hasWindows) {
  const winTargets = config.targets
    .filter((t) => t.rustTarget.includes("windows"))
    .map((t) => t.rustTarget);
  for (const wt of winTargets) {
    if (!installedTargets.includes(wt)) {
      throw new Error(
        `Rust target ${wt} not installed. Run: rustup target add ${wt}`,
      );
    }
  }
  console.log("  windows targets: OK");
}

for (const t of config.targets) {
  if (!installedTargets.includes(t.rustTarget)) {
    throw new Error(
      `Rust target ${t.rustTarget} not installed. Run: rustup target add ${t.rustTarget}`,
    );
  }
}
console.log("  all rust targets: OK\n");

// --- Snapshot everything we might mutate -----------------------------------

const fileSnapshots = new Map();
for (const f of [mainPkgPath, mainReadmePath]) {
  if (existsSync(f)) fileSnapshots.set(f, readFileSync(f));
}

function snapshotDir(dir) {
  const existed = existsSync(dir);
  const entries = new Map();
  if (!existed) return { existed, entries };
  const queue = [""];
  while (queue.length) {
    const rel = queue.shift();
    const full = rel ? path.join(dir, rel) : dir;
    const st = statSync(full);
    if (st.isDirectory()) {
      for (const child of readdirSync(full)) {
        queue.push(rel ? `${rel}/${child}` : child);
      }
    } else {
      entries.set(rel, readFileSync(full));
    }
  }
  return { existed, entries };
}

const distSnapshot = snapshotDir(distDir);
const platformsSnapshot = snapshotDir(platformsDir);

function restoreDir(dir, snapshot) {
  rmSync(dir, { recursive: true, force: true });
  if (!snapshot.existed) {
    // Directory did not exist before rehearsal — leave it deleted.
    return;
  }
  mkdirSync(dir, { recursive: true });
  for (const [rel, content] of snapshot.entries) {
    const full = path.join(dir, rel);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, content);
  }
}

let restored = false;

function restore() {
  if (restored) return;
  restored = true;
  for (const [filePath, content] of fileSnapshots) {
    writeFileSync(filePath, content);
  }
  restoreDir(distDir, distSnapshot);
  restoreDir(platformsDir, platformsSnapshot);
  // Remove the isolated build directory entirely — never touched the real target/.
  rmSync(isolatedTargetDir, { recursive: true, force: true });
}

// Node.js does not execute finally blocks on SIGINT. Register an explicit
// handler so the workspace is always restored, even on Ctrl+C.
process.on("SIGINT", () => {
  restore();
  process.exit(130);
});

// --- Step 1: Build ---------------------------------------------------------
console.log("=== Step 1: Build ===\n");
mkdirSync(distDir, { recursive: true });

try {
  for (const target of config.targets) {
    const rt = target.rustTarget;
    console.log(`Building ${rt}...`);
    const isWindows = rt.includes("windows");
    const isLinux = rt.includes("linux");
    const binaryName = `${cliName}${isWindows ? ".exe" : ""}`;
    const outDir = path.join(distDir, rt);
    mkdirSync(outDir, { recursive: true });

    const buildArgs = ["build", "--release", "--target", rt];
    if (isLinux) {
      buildArgs[0] = "zigbuild";
    }
    const buildResult = spawnSync("cargo", buildArgs, {
      stdio: "inherit",
      env: { ...process.env, CARGO_TARGET_DIR: isolatedTargetDir },
      shell: true,
    });
    if (buildResult.status !== 0) {
      throw new Error(
        `cargo build failed for ${rt} (exit ${buildResult.status}).`,
      );
    }

    const src = path.join(isolatedTargetDir, rt, "release", binaryName);
    if (!existsSync(src)) {
      throw new Error(`Built binary not found at ${src}.`);
    }
    const dst = path.join(outDir, binaryName);
    copyFileSync(src, dst);
    console.log(`  -> ${dst}`);
  }

  // --- Step 2: Sync platform packages ----------------------------------------
  console.log("\n=== Step 2: Sync platform packages ===\n");
  const syncResult = spawnSync(
    process.execPath,
    ["scripts/release/sync-platform-packages.mjs", rehearsalVersion],
    { cwd: rootDir, stdio: "inherit" },
  );
  if (syncResult.status !== 0) {
    throw new Error(
      `sync-platform-packages failed (exit ${syncResult.status}).`,
    );
  }

  // --- Step 3: npm publish --dry-run for every package -----------------------
  console.log("\n=== Step 3: npm publish --dry-run ===\n");

  for (const target of config.targets) {
    const pkgDir = path.join(platformsDir, target.packageSuffix);
    if (!existsSync(pkgDir)) {
      throw new Error(`Platform package missing: ${pkgDir}`);
    }
    const name = JSON.parse(
      readFileSync(path.join(pkgDir, "package.json"), "utf8"),
    ).name;
    console.log(`  ${name}@${rehearsalVersion}`);
    const r = spawnSync("npm", ["publish", "--dry-run", "--access=public"], {
      cwd: pkgDir,
      stdio: "inherit",
      shell: true,
    });
    if (r.status !== 0) {
      throw new Error(
        `npm publish --dry-run failed for ${name} (exit ${r.status}).`,
      );
    }
  }

  // Main package
  console.log(`  ${pkgName}@${rehearsalVersion}`);
  const mainR = spawnSync("npm", ["publish", "--dry-run", "--access=public"], {
    cwd: path.join(rootDir, "npm/main"),
    stdio: "inherit",
    shell: true,
  });
  if (mainR.status !== 0) {
    throw new Error(
      `npm publish --dry-run failed for main package (exit ${mainR.status}).`,
    );
  }

  console.log(
    "\n=== Rehearsal complete. No tags, no npm publishes, no GitHub Release. ===\n",
  );
} finally {
  restore();
}
