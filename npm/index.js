import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, normalize, relative, resolve } from 'node:path'

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

const parseOptions = (input = {}) => {
  const isBuild = process.env.NODE_ENV === 'production'
  const debug = input.debug ?? !isBuild
  const optimize = typeof input.optimize === 'boolean' ? input.optimize : !debug && isBuild
  const compilerOptions = input.compiler
    ? input.compiler
    : {
        debug,
        optimize,
        verbose: isBuild,
        pathToElm: 'elm',
        ...(input.nodeElmCompilerOptions ?? {}),
      }

  return {
    isBuild,
    compiler: input.compiler ?? null,
    compilerOptions,
  }
}

let queue = Promise.resolve()
const withLock = (fn) => {
  const next = queue.then(fn, fn)
  queue = next.catch(() => {})
  return next
}

const runRust = (command, args, input) =>
  new Promise((resolve, reject) => {
    const child = spawn(binary, [command, ...args], { stdio: ['pipe', 'pipe', 'pipe'] })
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
    if (typeof input === 'string') {
      child.stdin.end(input)
    } else {
      child.stdin.end()
    }
  })

const optionalArg = (value) => (value === undefined || value === null ? '-' : String(value))
const optionalObjectArg = (value) => (value === undefined || value === null ? '-' : JSON.stringify(value))

export const plugin = (userOptions = {}) => {
  const options = parseOptions(userOptions)
  const compilableFiles = new Map()

  return {
    name: 'vite-plugin-elm',
    enforce: 'pre',
    handleHotUpdate({ file, server, modules }) {
      const { valid } = parseImportId(file)
      if (!valid) return

      const modulesToCompile = []
      compilableFiles.forEach((dependencies, compilableFile) => {
        if (dependencies.has(normalize(file))) {
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
      let importer = ''
      for (const moduleId of this.getModuleIds()) {
        if (moduleId === id) break
        importer = moduleId
      }
      for (const accompany of withParams) {
        const resolved = await this.resolve(accompany, importer)
        if (resolved?.id) accompanies.push(resolved.id)
      }
      const targets = [pathname, ...accompanies].filter((target) => target !== '')
      const cwd = optionalArg(options.compilerOptions.cwd)

      let result
      if (options.compiler) {
        const dependencies = (await withLock(() => runRust('deps', [cwd, ...targets]))).map(normalize)
        const compiled = await options.compiler.compile(targets)
        const postprocessed = await withLock(() =>
          runRust(
            'postprocess',
            [String(options.isBuild), cwd, String(dependencies.length), ...dependencies],
            compiled,
          ),
        )
        result = {
          code: postprocessed.code,
          dependencies,
        }
        compilableFiles.set(id, new Set([...accompanies.map(normalize), ...dependencies]))
      } else {
        result = await withLock(() =>
          runRust('load', [
            String(options.isBuild),
            optionalArg(options.compilerOptions.debug),
            optionalArg(options.compilerOptions.optimize),
            optionalArg(options.compilerOptions.verbose),
            optionalArg(options.compilerOptions.pathToElm),
            optionalArg(options.compilerOptions.cwd),
            optionalArg(options.compilerOptions.report),
            optionalArg(options.compilerOptions.docs),
            optionalObjectArg(options.compilerOptions.processOpts),
            ...targets,
          ]),
        )
        result.dependencies = result.dependencies.map(normalize)
        compilableFiles.set(id, new Set([...accompanies.map(normalize), ...result.dependencies]))
      }

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
