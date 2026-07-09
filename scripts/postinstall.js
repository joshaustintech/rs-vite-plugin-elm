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

if (!hasRust || !hasElm) {
  console.error('\n========================================================================')
  console.error('ERROR: rs-vite-plugin-elm installation check failed.')
  if (!hasRust) {
    console.error('- Rust (cargo) is missing from your PATH.')
    console.error('  Please install Rust: https://www.rust-lang.org/')
  }
  if (!hasElm) {
    console.error('- Elm is missing from your PATH.')
    console.error('  Please install Elm: https://elm-lang.org/')
  }
  console.error('========================================================================\n')
  // We do not crash the npm installation process hard so that npm install doesn't display a large generic stack trace.
  // Instead, the plugin will check again and throw a clear, friendly error at runtime when Vite starts.
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
