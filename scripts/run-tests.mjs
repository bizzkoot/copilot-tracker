#!/usr/bin/env node
/* eslint-env node */

/**
 * Cross-platform test runner script.
 *
 * npm scripts on Windows run through PowerShell/cmd.exe, which do not expand
 * glob patterns like `*.test.mjs`. Node.js v20 `--test` flag also does NOT
 * perform its own glob expansion when the argument is received literally.
 *
 * This script uses Node.js built-in glob (available since v22+) or falls back
 * to reading the directory directly, then spawns `node --test` with explicit
 * file paths — no shell glob expansion required.
 */

import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(scriptDir, '..');
const testDir = join(repoRoot, 'tests', 'regression');

if (!existsSync(testDir)) {
  console.error(`Test directory not found: ${testDir}`);
  process.exit(1);
}

// Read all .test.mjs files from the regression directory
const testFiles = readdirSync(testDir)
  .filter((f) => f.endsWith('.test.mjs'))
  .sort()
  .map((f) => join(testDir, f));

if (testFiles.length === 0) {
  console.error('No test files found in', testDir);
  process.exit(1);
}

console.log(`Running ${testFiles.length} test files...\n`);

try {
  execFileSync(process.execPath, ['--test', ...testFiles], {
    stdio: 'inherit',
    cwd: repoRoot,
  });
} catch (e) {
  // node --test already printed failures; exit with its code
  process.exit(e.status ?? 1);
}
