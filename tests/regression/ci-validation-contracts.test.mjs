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
