import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

if (process.env.SEMVER_BUMP_IN_PROGRESS === '1') {
  process.exit(0);
}

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '..');

// Get the latest commit message
let commitMsg = '';
try {
  commitMsg = execSync('git log -1 --pretty=%B', { encoding: 'utf8', cwd: root }).trim();
} catch (e) {
  console.error('Failed to get git commit message:', e.message);
  process.exit(0);
}

// Ignore if it's already a semver bump or release commit
if (
  commitMsg.startsWith('chore(release):') ||
  commitMsg.includes('[skip ci]') ||
  commitMsg.includes('version bump')
) {
  process.exit(0);
}

// Determine bump type:
// Major: contains "BREAKING CHANGE:" or "BREAKING CHANGES:" or a "feat!:" prefix
// Minor: starts with "feat:" or similar features pattern
// Patch: default (every other commit)
let bumpType = 'patch';
if (
  commitMsg.includes('BREAKING CHANGE:') ||
  commitMsg.includes('BREAKING CHANGES:') ||
  /\w+!(\([^)]+\))?:/.test(commitMsg)
) {
  bumpType = 'major';
} else if (/^feat(\([^)]+\))?:/.test(commitMsg)) {
  bumpType = 'minor';
}

// Read current version from package.json
const pkgPath = join(root, 'package.json');
if (!existsSync(pkgPath)) {
  console.error('package.json not found');
  process.exit(1);
}
const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
const currentVersion = pkg.version;
if (!currentVersion) {
  console.error('No version found in package.json');
  process.exit(1);
}

const parts = currentVersion.split('.').map(Number);
if (parts.length !== 3 || parts.some(isNaN)) {
  console.error(`Invalid version format: ${currentVersion}`);
  process.exit(1);
}

let [major, minor, patch] = parts;
if (bumpType === 'major') {
  major += 1;
  minor = 0;
  patch = 0;
} else if (bumpType === 'minor') {
  minor += 1;
  patch = 0;
} else {
  patch += 1;
}

const nextVersion = `${major}.${minor}.${patch}`;
console.log(`Bumping version from ${currentVersion} to ${nextVersion} (${bumpType})`);

// Write package.json
pkg.version = nextVersion;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');

// Write Cargo.toml
const cargoPath = join(root, 'Cargo.toml');
if (existsSync(cargoPath)) {
  const cargoContent = readFileSync(cargoPath, 'utf8');
  const updatedCargo = cargoContent.replace(/^version\s*=\s*"[^"]*"/m, `version = "${nextVersion}"`);
  writeFileSync(cargoPath, updatedCargo, 'utf8');
}

// Run cargo check to update Cargo.lock if possible
try {
  execSync('cargo check', { cwd: root, stdio: 'ignore' });
} catch (e) {
  console.error('cargo check failed, Cargo.lock might not be updated:', e.message);
}

// Stage version files and amend commit
try {
  // Only stage files if they exist and are modified
  const filesToStage = ['package.json', 'Cargo.toml', 'Cargo.lock'].filter(file => existsSync(join(root, file)));
  if (filesToStage.length > 0) {
    execSync(`git add ${filesToStage.join(' ')}`, { cwd: root, stdio: 'inherit' });
    execSync('git commit --amend --no-edit', {
      cwd: root,
      stdio: 'inherit',
      env: { ...process.env, SEMVER_BUMP_IN_PROGRESS: '1' }
    });
  }
} catch (e) {
  console.error('Failed to amend commit with version bump:', e.message);
  process.exit(1);
}
