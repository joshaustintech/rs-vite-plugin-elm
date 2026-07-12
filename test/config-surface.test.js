import assert from 'node:assert/strict'
import { mkdtemp, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test from 'node:test'

import plugin from '../npm/index.js'

const compilerOptions = [
  {},
  { cwd: '/tmp/project' },
  { docs: 'docs.json' },
  { debug: true },
  { optimize: true },
  { processOpts: { env: 'ignored-for-compileToString' } },
  { report: 'json' },
  { pathToElm: 'elm-0.19.1' },
  { verbose: true },
  {
    cwd: '/tmp/project', docs: 'docs.json', debug: false, optimize: true,
    processOpts: { stdio: 'pipe' }, report: 'json', pathToElm: 'elm', verbose: false,
  },
]

test('accepts every documented built-in compiler configuration', () => {
  for (const debug of [undefined, true, false]) {
    for (const optimize of [undefined, true, false]) {
      for (const nodeElmCompilerOptions of compilerOptions) {
        const configured = plugin({ debug, optimize, nodeElmCompilerOptions })
        assert.equal(configured.name, 'vite-plugin-elm')
        assert.equal(configured.enforce, 'pre')
      }
    }
  }
})

test('custom compiler receives Elm targets and postprocessing remains active', async () => {
  const root = await mkdtemp(join(tmpdir(), 'rs-vite-plugin-elm-config-'))
  const main = join(root, 'Main.elm')
  await writeFile(join(root, 'elm.json'), '{"type":"application","source-directories":["."],"elm-version":"0.19.1","dependencies":{"direct":{},"indirect":{}},"test-dependencies":{"direct":{},"indirect":{}}}')
  await writeFile(main, 'module Main exposing (main)\n\nimport Html\n\nmain = Html.text "ok"\n')
  const calls = []
  const configured = plugin({
    compiler: {
      async compile(targets) {
        calls.push(targets)
        return 'export const Elm = { Main: { init: () => {} } };'
      },
    },
  })
  const output = await configured.load.call({ getModuleIds: function* () {}, resolve: async () => null }, main)

  assert.deepEqual(calls, [[main]])
  assert.match(output.code, /import\.meta\.hot/)
  assert.equal(output.map, null)
})
