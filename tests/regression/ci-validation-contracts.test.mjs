import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const packageJson = JSON.parse(
  fs.readFileSync(path.resolve("package.json"), "utf8"),
);
const prChecksSource = fs.readFileSync(
  path.resolve(".github/workflows/pr-checks.yml"),
  "utf8",
);
const releaseWorkflowSource = fs.readFileSync(
  path.resolve(".github/workflows/release.yml"),
  "utf8",
);
const manualReleasePublishWorkflowPath = path.resolve(
  ".github/workflows/manual-release-publish.yml",
);
const manualReleasePublishSource = fs.existsSync(
  manualReleasePublishWorkflowPath,
)
  ? fs.readFileSync(manualReleasePublishWorkflowPath, "utf8")
  : "";
const runTestsSource = fs.readFileSync(
  path.resolve("scripts/run-tests.mjs"),
  "utf8",
);

test("package scripts expose non-mutating CI lint validation", () => {
  assert.equal(
    packageJson.scripts["test:js"],
    "node scripts/run-tests.mjs",
    "CI should use the explicit cross-platform JS test runner",
  );
  assert.equal(
    packageJson.scripts["lint:js:check"],
    "eslint . --ext .js,.jsx,.cjs,.mjs,.ts,.tsx,.cts,.mts",
    "CI should use a non-mutating JS lint script",
  );
  assert.equal(
    packageJson.scripts["lint:ci"],
    "npm run lint:js:check && npm run lint:rust",
    "CI should use a dedicated non-mutating lint command",
  );
});

test("cross-platform JS test runner resolves files without relying on shell glob expansion", () => {
  assert.match(
    runTestsSource,
    /existsSync\(testDir\)/,
    "test runner should fail clearly when the regression test directory is missing",
  );
  assert.match(
    runTestsSource,
    /readdirSync\(testDir\)/,
    "test runner should enumerate test files directly",
  );
  assert.match(
    runTestsSource,
    /execFileSync\(process\.execPath,\s*\['--test',\s*\.\.\.testFiles\]/,
    "test runner should invoke Node directly with explicit test file paths",
  );
});

for (const [name, source] of [
  ["PR checks", prChecksSource],
  ["release validation", releaseWorkflowSource],
]) {
  test(`${name} runs repository tests and non-mutating lint validation`, () => {
    assert.match(
      source,
      /run:\s*npm run test\b/,
      `${name} workflow should run the repository test command`,
    );
    assert.match(
      source,
      /run:\s*npm run lint:ci\b/,
      `${name} workflow should use non-mutating lint validation`,
    );
    assert.doesNotMatch(
      source,
      /run:\s*npm run lint\s*(?:\r?\n|$)/,
      `${name} workflow should not run the mutating lint command`,
    );
  });
}

test("PR checks summary job does not create an extra failing job", () => {
  assert.doesNotMatch(
    prChecksSource,
    /name:\s*Fail workflow if validation failed/,
    "PR checks summary should report status without adding a redundant failure job",
  );
});

test("release validation uses bash for bash-specific sync checks and avoids redundant summary failures", () => {
  assert.match(
    releaseWorkflowSource,
    /- name:\s*Verify version sync[\s\S]*?shell:\s*bash/,
    "release validation should run bash-specific version checks with a bash shell",
  );
  assert.match(
    releaseWorkflowSource,
    /- name:\s*Report validation status[\s\S]*?shell:\s*bash/,
    "release validation status reporting should use a bash shell on Windows",
  );
  assert.doesNotMatch(
    releaseWorkflowSource,
    /name:\s*Fail workflow if validation failed/,
    "release summary should not add a redundant failing job",
  );
});

test("release workflow uses committed release-please configuration", () => {
  assert.match(
    releaseWorkflowSource,
    /config-file:\s*\.release-please\.json\b/,
    "release workflow should point release-please at the committed config file",
  );
  assert.doesNotMatch(
    releaseWorkflowSource,
    /config-file:\s*\.release-please-generated\.json\b/,
    "release workflow should not reference a runtime-generated config file",
  );
  assert.doesNotMatch(
    releaseWorkflowSource,
    /- name:\s*Resolve bootstrap SHA\b/,
    "release workflow should not depend on a bootstrap SHA step for established releases",
  );
  assert.doesNotMatch(
    releaseWorkflowSource,
    /- name:\s*Generate release-please config\b/,
    "release workflow should not generate a release-please config that the action cannot read",
  );
});

test("release workflow uses the release-please CLI for exact manual release-as PRs", () => {
  assert.match(
    releaseWorkflowSource,
    /skip-github-pull-request:\s*true/,
    "release workflow should keep the action limited to release tagging when using the CLI for PR creation",
  );
  assert.match(
    releaseWorkflowSource,
    /npx --yes release-please@17\.4\.1 release-pr/,
    "release workflow should use the release-please CLI for manifest PR creation",
  );
  assert.match(
    releaseWorkflowSource,
    /--config-file=\.release-please\.json/,
    "release workflow should keep the CLI on the committed release-please config file",
  );
  assert.match(
    releaseWorkflowSource,
    /--manifest-file=\.release-please-manifest\.json/,
    "release workflow should keep the CLI on the committed manifest file",
  );
  assert.match(
    releaseWorkflowSource,
    /--release-as="\$\{\{ steps\.parse-version\.outputs\.version \}\}"/,
    "release workflow should pass the parsed exact version to the CLI release-pr command",
  );
  assert.match(
    releaseWorkflowSource,
    /pr_created:\s*\$\{\{ steps\.release-pr-metadata\.outputs\.pr_created \}\}/,
    "release workflow should derive pr_created from explicit release PR metadata rather than simple PR existence",
  );
  assert.doesNotMatch(
    releaseWorkflowSource,
    /pr_created:\s*\$\{\{ steps\.resolve-release-pr\.outputs\.pr_found \}\}/,
    "release workflow should not treat any discovered open PR as newly created or updated",
  );
  assert.match(
    releaseWorkflowSource,
    /--target-branch="\$\{\{ github\.ref_name \}\}"/,
    "release workflow should target the same branch that triggered the manual run",
  );
  assert.doesNotMatch(
    releaseWorkflowSource,
    /--target-branch="main"/,
    "release workflow should not hardcode main for CLI release PR creation",
  );
  assert.doesNotMatch(
    releaseWorkflowSource,
    /base:\s*'main'/,
    "release workflow should not hardcode main when resolving release PRs",
  );
});

test("release workflow can publish automatically on merged release commits to main", () => {
  assert.match(
    releaseWorkflowSource,
    /push:\s*[\s\S]*?branches:\s*[\s\S]*?- main/,
    "release workflow should listen to pushes on main for release publishing",
  );
  assert.match(
    releaseWorkflowSource,
    /startsWith\(github\.event\.head_commit\.message, 'chore\(main\): release '\)/,
    "release workflow should only auto-publish when the pushed commit is a merged release commit",
  );
  assert.match(
    releaseWorkflowSource,
    /needs\.release-please\.outputs\.pr_created == 'true'/,
    "release PR formatting and validation should only run when the current run actually created or updated a release PR",
  );
});

test("manual release publish workflow can publish an already-merged version from changelog notes", () => {
  assert.match(
    manualReleasePublishSource,
    /name:\s*Manual Publish Existing Release/,
    "manual release publish workflow should be present",
  );
  assert.match(
    manualReleasePublishSource,
    /workflow_dispatch:\s*[\s\S]*?inputs:\s*[\s\S]*?version:/,
    "manual release publish workflow should accept a version input",
  );
  assert.match(
    manualReleasePublishSource,
    /ref:\s*main/,
    "manual release publish workflow should publish from main",
  );
  assert.match(
    manualReleasePublishSource,
    /CHANGELOG\.md/,
    "manual release publish workflow should extract notes from CHANGELOG.md",
  );
  assert.match(
    manualReleasePublishSource,
    /gh release create/,
    "manual release publish workflow should create the GitHub release directly",
  );
  assert.match(
    manualReleasePublishSource,
    /--notes-file\s+release_notes\.md/,
    "manual release publish workflow should publish the extracted changelog section as release notes",
  );
});
