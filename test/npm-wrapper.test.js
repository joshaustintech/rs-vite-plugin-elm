import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import test from 'node:test'

const here = dirname(fileURLToPath(import.meta.url))
const root = resolve(here, '..')
const runtimePath = '/opt/homebrew/bin:/usr/bin:/bin'

test('plugin loads when the packaged binary exists even if cargo is absent', () => {
  const result = spawnSync(
    process.execPath,
    [
      '--input-type=module',
      '-e',
      "import plugin from './npm/index.js'; plugin(); console.log('ok')",
    ],
    {
      cwd: root,
      env: {
        ...process.env,
        PATH: runtimePath,
      },
      encoding: 'utf8',
    },
  )

  assert.equal(result.status, 0, result.stderr)
  assert.match(result.stdout, /ok/)
})
