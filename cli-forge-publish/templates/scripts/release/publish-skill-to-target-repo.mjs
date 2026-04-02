import {
  existsSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import {
  archiveFilenameForTarget,
  checksumFilenameForArchive,
  computeSha256,
  copyDirectoryContents,
  detectPublicationMode,
  ensureCleanDir,
  ensureDir,
  getArtifactTarget,
  isLocalRepositoryTarget,
  loadReleaseConfig,
  normalizePath,
  readJson,
  relativeToRoot,
  releaseBuildBinaryPath,
  resolveDestinationBranch,
  resolveDestinationRepository,
  resolveLocalRepositoryPath,
  rootDir,
  runCommand,
  sharedSkillHomeRelativePath,
  sharedSkillReleaseMetadataRelativePath,
  targetArtifactsDir,
  targetBuildMetadataPath,
  writeJson,
} from "./release-helpers.mjs";

const [version, gitTag, gitHead] = process.argv.slice(2);

if (!version || !gitTag || !gitHead) {
  throw new Error(
    "Usage: node scripts/release/publish-skill-to-target-repo.mjs <version> <gitTag> <gitHead>",
  );
}

const config = loadReleaseConfig();
const destinationRepository = resolveDestinationRepository(config);
const destinationBranch = resolveDestinationBranch(config);
const publicationMode = detectPublicationMode(destinationRepository);

const publishRoot = path.join(rootDir, ".work/release/publish");
const destinationStateDir = path.join(
  rootDir,
  ".work/release/destination-state",
);
const skillHomeRelativePath = sharedSkillHomeRelativePath(config);
const releaseMetadataRelativePath =
  sharedSkillReleaseMetadataRelativePath(config);
const releaseMetadataPath = path.join(
  publishRoot,
  config.sharedRepository.skillRootPrefix,
  config.sourceSkillId,
  config.artifactBuild.releaseMetadataFilename,
);
const receiptPath = path.join(
  rootDir,
  ".work/release/last-publication-receipt.json",
);
const sourceRepository =
  process.env.GITHUB_REPOSITORY ||
  process.env.SKILL_RELEASE_SOURCE_REPOSITORY ||
  "unknown/unknown";
const sourceReleaseReference =
  sourceRepository === "unknown/unknown"
    ? gitTag
    : `https://github.com/${sourceRepository}/releases/tag/${gitTag}`;
const publishedAt = new Date().toISOString();

function hydrateRemoteDestinationState() {
  const destinationToken = process.env[config.destinationTokenEnv];
  if (!destinationToken) {
    throw new Error(
      `Missing ${config.destinationTokenEnv}; cannot hydrate the shared destination repository.`,
    );
  }

  const remoteUrl = `https://x-access-token:${encodeURIComponent(
    destinationToken,
  )}@github.com/${destinationRepository}.git`;

  const branchProbe = runCommand(
    "git",
    ["ls-remote", "--heads", remoteUrl, destinationBranch],
    {
      cwd: rootDir,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  if (!branchProbe.trim()) {
    ensureCleanDir(destinationStateDir);
    return;
  }

  rmSync(destinationStateDir, { recursive: true, force: true });
  runCommand("git", [
    "clone",
    "--depth",
    "1",
    "--branch",
    destinationBranch,
    remoteUrl,
    destinationStateDir,
  ]);
}

function hydrateDestinationState() {
  ensureCleanDir(destinationStateDir);

  if (isLocalRepositoryTarget(destinationRepository)) {
    const localDestinationPath = resolveLocalRepositoryPath(
      destinationRepository,
    );

    if (!existsSync(localDestinationPath)) {
      return;
    }

    if (!statSync(localDestinationPath).isDirectory()) {
      throw new Error(
        `Local destination repository target is not a directory: ${localDestinationPath}.`,
      );
    }

    copyDirectoryContents(localDestinationPath, destinationStateDir, {
      exclude: [".git"],
    });
    return;
  }

  hydrateRemoteDestinationState();
}

function loadExistingCatalog() {
  const catalogPath = path.join(
    destinationStateDir,
    config.sharedRepository.catalogPath,
  );

  if (!existsSync(catalogPath)) {
    return {
      entries: [],
      entry_count: 0,
      format: "json",
      path: config.sharedRepository.catalogPath,
      updated_at: null,
    };
  }

  const catalog = readJson(catalogPath);
  if (!Array.isArray(catalog.entries)) {
    throw new Error("Existing catalog.json is missing an entries array.");
  }

  return catalog;
}

function preservedSkillPathsFromState() {
  const skillsRoot = path.join(
    destinationStateDir,
    config.sharedRepository.skillRootPrefix,
  );

  if (!existsSync(skillsRoot)) {
    return [];
  }

  return readdirSync(skillsRoot)
    .filter((entry) => entry !== config.sourceSkillId)
    .map((entry) =>
      normalizePath(
        path.posix.join(config.sharedRepository.skillRootPrefix, entry),
      ),
    )
    .sort();
}

function ensureBuildMetadata(target) {
  let metadata = readJson(targetBuildMetadataPath(config, target));

  if (
    !metadata &&
    publicationMode === "rehearsal" &&
    process.env.SKILL_RELEASE_DISABLE_SYNTHETIC_ARTIFACTS !== "true"
  ) {
    runCommand(process.execPath, [
      path.join(rootDir, "scripts/release/build-cli-artifact.mjs"),
      target,
      "--synthetic",
    ]);
    metadata = readJson(targetBuildMetadataPath(config, target));
  }

  return metadata;
}

function packageArtifactForTarget(target, versionedSkillHomeStageDir) {
  const targetConfig = getArtifactTarget(config, target);
  const buildMetadata = ensureBuildMetadata(target);

  if (!buildMetadata) {
    return {
      artifactOrigin: null,
      archiveFilename: null,
      archivePath: null,
      blockingReason: `Missing prepared build output for ${target}.`,
      checksumFilename: null,
      checksumPath: null,
      publicationStatus: "failed",
      required: targetConfig.required !== false,
      targetVariant: target,
    };
  }

  if (
    publicationMode === "live_release" &&
    buildMetadata.artifactOrigin === "synthetic_rehearsal"
  ) {
    return {
      artifactOrigin: buildMetadata.artifactOrigin,
      archiveFilename: null,
      archivePath: null,
      blockingReason: `Live publication cannot use synthetic rehearsal artifacts for ${target}.`,
      checksumFilename: null,
      checksumPath: null,
      publicationStatus: "failed",
      required: targetConfig.required !== false,
      targetVariant: target,
    };
  }

  const binaryPath = path.join(rootDir, buildMetadata.binaryPath);
  if (!existsSync(binaryPath)) {
    return {
      artifactOrigin: buildMetadata.artifactOrigin,
      archiveFilename: null,
      archivePath: null,
      blockingReason: `Prepared binary for ${target} is missing: ${buildMetadata.binaryPath}.`,
      checksumFilename: null,
      checksumPath: null,
      publicationStatus: "failed",
      required: targetConfig.required !== false,
      targetVariant: target,
    };
  }

  const archiveFilename = archiveFilenameForTarget(config, version, target);
  const checksumFilename = checksumFilenameForArchive(archiveFilename);
  const archivePath = path.join(versionedSkillHomeStageDir, archiveFilename);
  const checksumPath = path.join(versionedSkillHomeStageDir, checksumFilename);

  runCommand("tar", [
    "-czf",
    archivePath,
    "-C",
    path.dirname(binaryPath),
    path.basename(binaryPath),
  ]);

  const sha256 = computeSha256(archivePath);
  writeFileSync(checksumPath, `${sha256}  ${archiveFilename}\n`, "utf8");

  return {
    archiveFilename,
    archivePath: normalizePath(
      path.posix.join(skillHomeRelativePath, archiveFilename),
    ),
    artifactOrigin: buildMetadata.artifactOrigin,
    binaryName: buildMetadata.binaryName,
    checksumFilename,
    checksumPath: normalizePath(
      path.posix.join(skillHomeRelativePath, checksumFilename),
    ),
    publicationStatus: "published",
    required: targetConfig.required !== false,
    sha256,
    targetVariant: target,
  };
}

function writeFailureEvidence({
  artifactResults,
  blockingReason,
  catalog,
  preservedSkillPaths,
  runResult,
}) {
  ensureDir(publishRoot);

  const manifest = {
    artifactResults,
    blockingReason,
    catalogPath: config.sharedRepository.catalogPath,
    catalogUpdated: false,
    catalog_updated: false,
    destinationBranch,
    destinationRepository,
    generatedPackageBoundary: config.generatedPackageBoundary,
    metadataVersion: 1,
    outputClassification: {
      finalDistributableArtifacts: [],
      repositoryOwnedPackagingEvidence: [
        config.metadataFilename,
        ".work/release/last-publication-receipt.json",
      ],
    },
    preservedSkillPaths,
    publicationMode,
    publishedAt,
    releaseMetadataPath: null,
    runResult,
    skillId: config.sourceSkillId,
    sourceCommitSha: gitHead,
    sourceGitTag: gitTag,
    sourceReleaseReference,
    sourceRepository,
    sourceVersion: version,
    updatedSkillPath: skillHomeRelativePath,
  };

  writeJson(path.join(publishRoot, config.metadataFilename), manifest);
  if (catalog?.entries) {
    writeJson(
      path.join(publishRoot, config.sharedRepository.catalogPath),
      catalog,
    );
  }

  const receipt = {
    artifactResults,
    blockingReason,
    catalogPath: config.sharedRepository.catalogPath,
    catalogUpdated: false,
    catalog_updated: false,
    destinationBranch,
    destinationRepository,
    publicationMode,
    publicationResult: "failed",
    publishRoot: relativeToRoot(publishRoot),
    publishedAt,
    releaseMetadataPath: null,
    runResult,
    sourceCommitSha: gitHead,
    sourceGitTag: gitTag,
    sourceReleaseReference,
    sourceSkillId: config.sourceSkillId,
    sourceVersion: version,
    updatedSkillPath: skillHomeRelativePath,
    preservedSkillPaths,
  };

  writeJson(receiptPath, receipt);
  process.stdout.write(`${JSON.stringify(receipt)}\n`);
  process.exit(1);
}

hydrateDestinationState();
ensureCleanDir(publishRoot);
copyDirectoryContents(destinationStateDir, publishRoot, { exclude: [".git"] });

const existingCatalog = loadExistingCatalog();
const preservedSkillPaths = preservedSkillPathsFromState();
const skillHomeStageDir = path.join(rootDir, ".work/release/skill-home-stage");

ensureCleanDir(skillHomeStageDir);

const artifactResults = config.artifactTargets.map((targetConfig) =>
  packageArtifactForTarget(targetConfig.target, skillHomeStageDir),
);

const failedRequiredArtifacts = artifactResults.filter(
  (entry) => entry.required && entry.publicationStatus !== "published",
);

if (failedRequiredArtifacts.length > 0) {
  writeFailureEvidence({
    artifactResults,
    blockingReason: failedRequiredArtifacts
      .map((entry) => entry.blockingReason)
      .join(" "),
    catalog: existingCatalog,
    preservedSkillPaths,
    runResult: "blocked",
  });
}

const skillHomePath = path.join(publishRoot, skillHomeRelativePath);
if (existsSync(skillHomePath) && !statSync(skillHomePath).isDirectory()) {
  writeFailureEvidence({
    artifactResults,
    blockingReason: `Target skill home path collides with a non-directory path: ${skillHomeRelativePath}.`,
    catalog: existingCatalog,
    preservedSkillPaths,
    runResult: "failed",
  });
}

rmSync(skillHomePath, { recursive: true, force: true });
ensureDir(skillHomePath);
copyDirectoryContents(skillHomeStageDir, skillHomePath);

const currentVersionArtifacts = artifactResults
  .filter((entry) => entry.publicationStatus === "published")
  .map((entry) => ({
    archive_filename: entry.archiveFilename,
    archive_path: entry.archivePath,
    artifact_origin: entry.artifactOrigin,
    binary_name: entry.binaryName,
    required: entry.required,
    target_variant: entry.targetVariant,
  }));

const currentVersionChecksums = artifactResults
  .filter((entry) => entry.publicationStatus === "published")
  .map((entry) => ({
    checksum_filename: entry.checksumFilename,
    checksum_path: entry.checksumPath,
    sha256: entry.sha256,
    target_variant: entry.targetVariant,
  }));

const releaseMetadata = {
  artifacts: currentVersionArtifacts,
  checksums: currentVersionChecksums,
  current_version: version,
  publication_path: skillHomeRelativePath,
  skill_id: config.sourceSkillId,
  source_commit_sha: gitHead,
  source_release_reference: sourceReleaseReference,
  source_release_tag: gitTag,
  synthetic_rehearsal_notice:
    publicationMode === "rehearsal"
      ? config.artifactBuild.syntheticRehearsalNotice
      : null,
  target_variants: currentVersionArtifacts.map((entry) => entry.target_variant),
  updated_at: publishedAt,
};

writeJson(releaseMetadataPath, releaseMetadata);

const catalogEntries = existingCatalog.entries
  .filter((entry) => entry.skill_id !== config.sourceSkillId)
  .concat({
    current_version: version,
    publication_path: skillHomeRelativePath,
    release_metadata_path: releaseMetadataRelativePath,
    skill_id: config.sourceSkillId,
    target_variants: releaseMetadata.target_variants,
    updated_at: publishedAt,
  })
  .sort((left, right) => left.skill_id.localeCompare(right.skill_id));

const updatedCatalog = {
  entries: catalogEntries,
  entry_count: catalogEntries.length,
  format: "json",
  path: config.sharedRepository.catalogPath,
  updated_at: publishedAt,
};

writeJson(
  path.join(publishRoot, config.sharedRepository.catalogPath),
  updatedCatalog,
);

const manifest = {
  artifactResults,
  catalogPath: config.sharedRepository.catalogPath,
  catalogUpdated: true,
  catalog_updated: true,
  destinationBranch,
  destinationRepository,
  generatedPackageBoundary: config.generatedPackageBoundary,
  metadataVersion: 1,
  outputClassification: {
    finalDistributableArtifacts: [
      skillHomeRelativePath,
      releaseMetadataRelativePath,
      config.sharedRepository.catalogPath,
    ],
    repositoryOwnedPackagingEvidence: [
      config.metadataFilename,
      ".work/release/last-publication-receipt.json",
    ],
  },
  preservedSkillPaths,
  publicationMode,
  publishedAt,
  releaseMetadataPath: releaseMetadataRelativePath,
  runResult: publicationMode === "live_release" ? "published" : "prepared",
  skillId: config.sourceSkillId,
  sourceCommitSha: gitHead,
  sourceGitTag: gitTag,
  sourceReleaseReference,
  sourceRepository,
  sourceVersion: version,
  updatedSkillPath: skillHomeRelativePath,
};

writeJson(path.join(publishRoot, config.metadataFilename), manifest);

const receipt = {
  artifactResults,
  catalogPath: config.sharedRepository.catalogPath,
  catalogUpdated: true,
  catalog_updated: true,
  destinationBranch,
  destinationRepository,
  publicationMode,
  publicationResult:
    publicationMode === "live_release" ? "published" : "prepared",
  publishRoot: relativeToRoot(publishRoot),
  publishedAt,
  releaseMetadataPath: releaseMetadataRelativePath,
  runResult: publicationMode === "live_release" ? "published" : "prepared",
  sourceCommitSha: gitHead,
  sourceGitTag: gitTag,
  sourceReleaseReference,
  sourceSkillId: config.sourceSkillId,
  sourceVersion: version,
  updatedSkillPath: skillHomeRelativePath,
  preservedSkillPaths,
};

writeJson(receiptPath, receipt);
process.stdout.write(`${JSON.stringify(receipt)}\n`);
