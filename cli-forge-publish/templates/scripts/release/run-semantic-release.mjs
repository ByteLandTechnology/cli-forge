import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import {
  loadReleaseConfig,
  resolveDestinationBranch,
  resolveDestinationRepository,
  rootDir,
  runCommand,
  writeJson,
} from "./release-helpers.mjs";

const semanticReleaseArgs = [
  "--yes",
  "-p",
  "semantic-release@25",
  "-p",
  "@semantic-release/changelog@6",
  "-p",
  "@semantic-release/commit-analyzer@13",
  "-p",
  "@semantic-release/exec@7",
  "-p",
  "@semantic-release/github@12",
  "-p",
  "@semantic-release/npm@13",
  "-p",
  "@semantic-release/release-notes-generator@14",
  "-p",
  "conventional-changelog-conventionalcommits@9",
  "semantic-release",
  ...process.argv.slice(2),
];

const result = spawnSync("npx", semanticReleaseArgs, {
  cwd: rootDir,
  env: process.env,
  encoding: "utf8",
});

if (result.stdout) {
  process.stdout.write(result.stdout);
}

if (result.stderr) {
  process.stderr.write(result.stderr);
}

const config = loadReleaseConfig();
const receiptPath = `${rootDir}/.work/release/last-publication-receipt.json`;

if (!existsSync(receiptPath)) {
  const combinedOutput = [result.stdout ?? "", result.stderr ?? ""].join("\n");
  const noReleaseDetected =
    combinedOutput.includes(
      "There are no relevant changes, so no new version is released.",
    ) || combinedOutput.includes("Found 0 commits since last release");

  const destinationRepository = (() => {
    try {
      return resolveDestinationRepository(config);
    } catch {
      return (
        process.env[config.destinationRepositoryEnv] ||
        config.destinationRepository
      );
    }
  })();

  const receipt = {
    artifactResults: [],
    blockingReason:
      result.status === 0
        ? noReleaseDetected
          ? "semantic-release found no releasable changes"
          : "semantic-release completed without preparing a destination publication receipt"
        : "semantic-release failed before destination publication preparation",
    catalogPath: config.sharedRepository.catalogPath,
    catalogUpdated: false,
    destinationBranch: resolveDestinationBranch(config),
    destinationRepository,
    publicationMode: process.argv.includes("--dry-run")
      ? "dry_run"
      : process.env.GITHUB_ACTIONS === "true"
        ? "live_release"
        : "report_only",
    publicationResult: result.status === 0 ? "skipped" : "failed",
    publishedAt: new Date().toISOString(),
    publishRoot: ".work/release/publish",
    releaseMetadataPath: null,
    runResult:
      result.status === 0
        ? noReleaseDetected
          ? "no_release"
          : "prepared"
        : "failed",
    sourceCommitSha: runCommand("git", ["rev-parse", "HEAD"], {
      cwd: rootDir,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim(),
    sourceGitTag: null,
    sourceSkillId: config.sourceSkillId,
    sourceVersion: null,
    updatedSkillPath: null,
  };

  writeJson(receiptPath, receipt);
}

if ((result.status ?? 0) !== 0) {
  process.exit(result.status ?? 1);
}
