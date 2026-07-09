import { spawnSync } from 'node:child_process'
import { copyFileSync, mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { platform } from 'node:os'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')

console.log('Building rs-vite-plugin-elm Rust binary...')
const cargoResult = spawnSync('cargo', ['build', '--release'], {
  cwd: root,
  stdio: 'inherit',
})

if (cargoResult.status !== 0) {
  console.error('Cargo build failed.')
  process.exit(cargoResult.status ?? 1)
}

const isWindows = platform() === 'win32'
const binaryName = isWindows ? 'rs-vite-plugin-elm.exe' : 'rs-vite-plugin-elm'

const distDir = join(root, 'dist')
const binDir = join(root, 'bin')

mkdirSync(distDir, { recursive: true })
mkdirSync(binDir, { recursive: true })

const srcBinary = join(root, 'target', 'release', binaryName)
const destBinary = join(binDir, binaryName)

console.log(`Copying binary from ${srcBinary} to ${destBinary}...`)
copyFileSync(srcBinary, destBinary)

console.log('Copying JS assets...')
copyFileSync(join(root, 'npm', 'index.js'), join(distDir, 'index.js'))
copyFileSync(join(root, 'npm', 'index.d.ts'), join(distDir, 'index.d.ts'))

console.log('Build completed successfully.')
