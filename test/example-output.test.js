import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import crypto from 'node:crypto'
import { readFile } from 'node:fs/promises'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import test from 'node:test'

const here = dirname(fileURLToPath(import.meta.url))
const fixture = JSON.parse(await readFile(resolve(here, 'example-output.snapshot.json'), 'utf8'))
const exampleRoot = resolve(here, '../../vite-plugin-elm/example')
const pluginPath = resolve(here, '../../vite-plugin-elm/dist/index.js')

const loadOutput = async (id, context = {}) => {
  const script = `
    import crypto from 'node:crypto'
    import plugin from ${JSON.stringify(pluginPath)}
    const p = plugin()
    const ctx = ${context.with ? `{
      addWatchFile() {},
      getModuleIds() {
        return ${JSON.stringify(context.getModuleIds)}
      },
      resolve(specifier, importer) {
        return { id: new URL(specifier, 'file://' + importer).pathname }
      },
    }` : `{ addWatchFile() {} }`}
    const out = await p.load.call(ctx, ${JSON.stringify(id)})
    console.log(JSON.stringify({
      hash: crypto.createHash('sha256').update(out.code).digest('hex'),
      codeLength: out.code.length,
    }))
  `
  const result = spawnSync(process.execPath, ['--input-type=module', '-e', script], {
    cwd: exampleRoot,
    encoding: 'utf8',
  })
  assert.equal(result.status, 0, result.stderr)
  return JSON.parse(result.stdout)
}

test('freezes the upstream example plugin outputs', async () => {
  const hello = await loadOutput('/Users/josh/vite_plugin_elm_work/vite-plugin-elm/example/src/Hello.elm')
  const application = await loadOutput('/Users/josh/vite_plugin_elm_work/vite-plugin-elm/example/src/Application.elm')
  const description = await loadOutput(
    '/Users/josh/vite_plugin_elm_work/vite-plugin-elm/example/src/Description.elm?with=./AnotherDescription.elm',
    {
      with: true,
      getModuleIds: [
        '/Users/josh/vite_plugin_elm_work/vite-plugin-elm/example/src/elements.js',
        '/Users/josh/vite_plugin_elm_work/vite-plugin-elm/example/src/Description.elm?with=./AnotherDescription.elm',
      ],
    },
  )

  assert.deepEqual(hello, fixture.Hello)
  assert.deepEqual(application, fixture.Application)
  assert.deepEqual(description, fixture['Description+AnotherDescription'])
})
