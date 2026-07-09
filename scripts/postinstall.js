import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')

function checkCommand(cmd) {
  try {
    const res = spawnSync(cmd, ['--version'], { stdio: 'ignore' })
    return res.status === 0
  } catch {
    return false
  }
}

const hasRust = checkCommand('cargo')
const hasElm = checkCommand('elm')
const binaryName = process.platform === 'win32' ? 'rs-vite-plugin-elm.exe' : 'rs-vite-plugin-elm'
const binary = resolve(root, 'bin', binaryName)

if (!hasElm) {
  console.error('\n========================================================================')
  console.error('ERROR: rs-vite-plugin-elm installation check failed.')
  if (!hasElm) {
    console.error('- Elm is missing from your PATH.')
    console.error('  Please install Elm: https://elm-lang.org/')
  }
  console.error('========================================================================\n')
  // We do not crash the npm installation process hard so that npm install doesn't display a large generic stack trace.
  // Instead, the plugin will check again and throw a clear, friendly error at runtime when Vite starts.
} else {
  if (existsSync(binary)) {
    console.log('Native plugin binary already present. Skipping rebuild.')
  } else {
    if (!hasRust) {
      console.error('\n========================================================================')
      console.error('ERROR: rs-vite-plugin-elm installation check failed.')
      console.error('- Rust (cargo) is missing from your PATH.')
      console.error('  Please install Rust: https://www.rust-lang.org/')
      console.error('========================================================================\n')
    } else {
      console.log('Rust and Elm found in PATH. Building native plugin binary...')
      const buildResult = spawnSync('node', [resolve(__dirname, 'build.js')], {
        cwd: root,
        stdio: 'inherit',
      })
      if (buildResult.status !== 0) {
        console.error('Failed to build native plugin binary.')
      }
    }
  }
}

// Setup git hooks if in a git repository
try {
  const isGit = spawnSync('git', ['rev-parse', '--is-inside-work-tree'], { stdio: 'ignore' });
  if (isGit.status === 0) {
    console.log('Configuring local git hooks directory...');
    const hooksRes = spawnSync('git', ['config', 'core.hooksPath', '.githooks'], {
      cwd: root,
      stdio: 'inherit',
    });
    if (hooksRes.status !== 0) {
      console.warn('Warning: Failed to configure git hooks path.');
    }
  }
} catch (e) {
  // Git command might not exist or failed
}
