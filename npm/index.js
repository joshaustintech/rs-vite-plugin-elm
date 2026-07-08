import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, relative, resolve } from 'node:path'

const here = dirname(fileURLToPath(import.meta.url))
const binary = resolve(here, '../bin/rs-vite-plugin-elm')

const trimDebugMessage = (code) => code.replace(/(console\.warn\('Compiled in DEBUG mode)/, '// $1')
const viteProjectPath = (dependency) => `/${relative(process.cwd(), dependency)}`

const parseImportId = (id) => {
  const parsedId = new URL(id, 'file://')
  const pathname = parsedId.pathname
  const valid = pathname.endsWith('.elm') && !parsedId.searchParams.has('raw')
  return {
    valid,
    pathname,
    withParams: parsedId.searchParams.getAll('with'),
  }
}

let queue = Promise.resolve()
const withLock = (fn) => {
  const next = queue.then(fn, fn)
  queue = next.catch(() => {})
  return next
}

const parseOptions = (input = {}) => {
  const isBuild = process.env.NODE_ENV === 'production'
  const debug = input.debug ?? !isBuild
  const optimize = typeof input.optimize === 'boolean' ? input.optimize : !debug && isBuild
  return {
    isBuild,
    debug,
    optimize,
    verbose: isBuild,
    pathToElm: input.nodeElmCompilerOptions?.pathToElm ?? '-',
  }
}

const runRust = (args) =>
  new Promise((resolve, reject) => {
    const child = spawn(binary, args, { stdio: ['ignore', 'pipe', 'pipe'] })
    let stdout = ''
    let stderr = ''
    child.stdout.setEncoding('utf8')
    child.stderr.setEncoding('utf8')
    child.stdout.on('data', (chunk) => {
      stdout += chunk
    })
    child.stderr.on('data', (chunk) => {
      stderr += chunk
    })
    child.on('error', reject)
    child.on('close', (code) => {
      if (code === 0) {
        resolve(JSON.parse(stdout))
      } else {
        reject(new Error(stderr || `rs-vite-plugin-elm exited with ${code}`))
      }
    })
  })

export const plugin = (userOptions = {}) => {
  const options = parseOptions(userOptions)
  const compilableFiles = new Map()

  return {
    name: 'rs-vite-plugin-elm',
    enforce: 'pre',
    handleHotUpdate({ file, server, modules }) {
      const { valid } = parseImportId(file)
      if (!valid) return

      const modulesToCompile = []
      compilableFiles.forEach((dependencies, compilableFile) => {
        if (dependencies.has(file)) {
          const module = server.moduleGraph.getModuleById(compilableFile)
          if (module) modulesToCompile.push(module)
        }
      })

      if (modulesToCompile.length > 0) {
        server.ws.send({
          type: 'custom',
          event: 'hot-update-dependents',
          data: modulesToCompile.map(({ url }) => url),
        })
        return modulesToCompile
      }
      return modules
    },
    async load(id) {
      const { valid, pathname, withParams } = parseImportId(id)
      if (!valid) return

      const accompanies = []
      for (const accompany of withParams) {
        const resolved = await this.resolve(accompany, id)
        if (resolved?.id) accompanies.push(resolved.id)
      }
      const targets = [pathname, ...accompanies]

      const result = await withLock(() =>
        runRust([
          'load',
          String(options.isBuild),
          String(options.debug),
          String(options.optimize),
          String(options.verbose),
          options.pathToElm,
          ...targets,
        ]),
      )

      compilableFiles.set(id, new Set([...accompanies, ...result.dependencies]))
      if (this.addWatchFile) {
        result.dependencies.forEach(this.addWatchFile.bind(this))
      }

      return {
        code: options.isBuild ? result.code : trimDebugMessage(result.code),
        map: null,
      }
    },
  }
}

export default plugin
