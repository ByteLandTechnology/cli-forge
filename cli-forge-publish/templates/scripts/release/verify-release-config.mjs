import { appendFileSync, existsSync } from "node:fs";
import path from "node:path";
import {
  getArtifactTarget,
  loadReleaseConfig,
  prepareGeneratedSkillProject,
  releaseArtifactsDir,
  requiredArtifactTargets,
  resolveDestinationBranch,
  resolveDestinationRepository,
  rootDir,
  sharedSkillHomeRelativePath,
  sharedSkillReleaseMetadataRelativePath,
  isLocalRepositoryTarget,
} from "./release-helpers.mjs";

const config = loadReleaseConfig();

function isUnreplacedPlaceholder(value) {
  return typeof value === "string" && value.includes("REPLACE_WITH_");
}

function verifyPlaceholderReplaced(value, fieldPath) {
  if (!value) {
    throw new Error(`${fieldPath} is required.`);
  }

  if (isUnreplacedPlaceholder(value)) {
    throw new Error(
      `${fieldPath} still contains a REPLACE_WITH_* placeholder. Replace all REPLACE_WITH_* placeholders in release/skill-release.config.json before running release automation.`,
    );
  }
}

function verifyDestinationToken(destinationRepository) {
  if (
    !isLocalRepositoryTarget(destinationRepository) &&
    !process.env[config.destinationTokenEnv]
  ) {
    throw new Error(
      [
        "Missing destination repository credential.",
        `Set ${config.destinationTokenEnv} before running the release workflow.`,
      ].join(" "),
    );
  }
}

function verifySharedRepositoryContract() {
  const { sharedRepository } = config;

  if (sharedRepository.catalogPath !== "catalog.json") {
    throw new Error("sharedRepository.catalogPath must be catalog.json.");
  }

  if (sharedRepository.skillRootPrefix !== "skills") {
    throw new Error("sharedRepository.skillRootPrefix must be skills.");
  }

  if (sharedRepository.retentionPolicy !== "current_only") {
    throw new Error("sharedRepository.retentionPolicy must be current_only.");
  }

  if (sharedRepository.updateMode !== "merged_tree_publish") {
    throw new Error("sharedRepository.updateMode must be merged_tree_publish.");
  }
}

function verifyGeneratedSkillConfig() {
  const { generatedSkill } = config;

  verifyPlaceholderReplaced(config.sourceSkillId, "sourceSkillId");
  verifyPlaceholderReplaced(
    generatedSkill.skillName,
    "generatedSkill.skillName",
  );
  verifyPlaceholderReplaced(
    generatedSkill.description,
    "generatedSkill.description",
  );
  verifyPlaceholderReplaced(generatedSkill.author, "generatedSkill.author");
  verifyPlaceholderReplaced(
    generatedSkill.projectPath,
    "generatedSkill.projectPath",
  );
  verifyPlaceholderReplaced(
    config.artifactBuild?.binaryName,
    "artifactBuild.binaryName",
  );

  if (generatedSkill.skillName !== config.sourceSkillId) {
    throw new Error(
      "generatedSkill.skillName must exist and match sourceSkillId.",
    );
  }

  if (
    !generatedSkill.templates ||
    Object.keys(generatedSkill.templates).length === 0
  ) {
    throw new Error(
      "generatedSkill.templates must map release fixture templates to output paths.",
    );
  }

  for (const templateRelativePath of Object.keys(generatedSkill.templates)) {
    const templatePath = path.join(rootDir, templateRelativePath);
    if (!existsSync(templatePath)) {
      throw new Error(
        `Missing configured release template: ${templateRelativePath}.`,
      );
    }
  }

  prepareGeneratedSkillProject(config);
}

function verifyArtifactTargets() {
  if (
    !Array.isArray(config.artifactTargets) ||
    config.artifactTargets.length === 0
  ) {
    throw new Error("artifactTargets must contain at least one entry.");
  }

  const requiredTargets = requiredArtifactTargets(config).map(
    (entry) => entry.target,
  );
  const minimumTargets = ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin"];

  for (const minimumTarget of minimumTargets) {
    if (!requiredTargets.includes(minimumTarget)) {
      throw new Error(`Missing required artifact target: ${minimumTarget}.`);
    }

    const targetConfig = getArtifactTarget(config, minimumTarget);
    if (!targetConfig.archiveFormat) {
      throw new Error(
        `Artifact target ${minimumTarget} must define archiveFormat.`,
      );
    }

    if (!targetConfig.runner) {
      throw new Error(`Artifact target ${minimumTarget} must define runner.`);
    }
  }
}

function verifyGeneratedPackageBoundary() {
  if (!config.generatedPackageBoundary) {
    throw new Error(
      "Missing generatedPackageBoundary in release/skill-release.config.json.",
    );
  }

  const { packageLocalSupportExamples, repositoryOwnedAutomation } =
    config.generatedPackageBoundary;

  if (
    !Array.isArray(packageLocalSupportExamples) ||
    packageLocalSupportExamples.length === 0
  ) {
    throw new Error(
      "generatedPackageBoundary.packageLocalSupportExamples must contain at least one entry.",
    );
  }

  if (
    !Array.isArray(repositoryOwnedAutomation) ||
    repositoryOwnedAutomation.length === 0
  ) {
    throw new Error(
      "generatedPackageBoundary.repositoryOwnedAutomation must contain at least one entry.",
    );
  }

  const missingAutomationPaths = repositoryOwnedAutomation.filter(
    (entry) => !existsSync(path.join(rootDir, entry)),
  );

  if (missingAutomationPaths.length > 0) {
    throw new Error(
      `Configured repository-owned automation paths do not exist: ${missingAutomationPaths.join(", ")}.`,
    );
  }
}

const destinationRepository = resolveDestinationRepository(config);
const destinationBranch = resolveDestinationBranch(config);

verifyDestinationToken(destinationRepository);
verifySharedRepositoryContract();
verifyGeneratedSkillConfig();
verifyArtifactTargets();
verifyGeneratedPackageBoundary();

if (process.env.GITHUB_OUTPUT) {
  appendFileSync(
    process.env.GITHUB_OUTPUT,
    [
      `catalog_path=${config.sharedRepository.catalogPath}`,
      `destination_branch=${destinationBranch}`,
      `destination_repository=${destinationRepository}`,
      `generated_skill_project=${config.generatedSkill.projectPath}`,
      `publish_dir=.work/release/publish`,
      `release_metadata_path=${sharedSkillReleaseMetadataRelativePath(config)}`,
      `required_targets=${requiredArtifactTargets(config)
        .map((entry) => entry.target)
        .join(",")}`,
      `skill_home_path=${sharedSkillHomeRelativePath(config)}`,
      `staged_artifacts_dir=${path.relative(rootDir, releaseArtifactsDir(config)).replace(/\\/g, "/")}`,
    ].join("\n") + "\n",
    "utf8",
  );
}

console.log(
  `Release destination configuration verified for ${destinationRepository} on ${destinationBranch}.`,
);
