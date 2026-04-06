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

test("package scripts expose non-mutating CI lint validation", () => {
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
