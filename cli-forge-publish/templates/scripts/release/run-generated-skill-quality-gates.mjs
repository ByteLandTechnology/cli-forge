import {
  loadReleaseConfig,
  prepareGeneratedSkillProject,
  runCommand,
} from "./release-helpers.mjs";

const config = loadReleaseConfig();
const projectDir = prepareGeneratedSkillProject(config);

runCommand("cargo", ["fmt"], { cwd: projectDir });
runCommand("cargo", ["fmt", "--check"], { cwd: projectDir });
runCommand("cargo", ["clippy", "--", "-D", "warnings"], { cwd: projectDir });
runCommand("cargo", ["test"], { cwd: projectDir });

console.log(
  `Target-project quality gates passed for ${config.generatedSkill.skillName} in ${projectDir}.`,
);
