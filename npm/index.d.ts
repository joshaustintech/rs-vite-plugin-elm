import type { Plugin } from 'vite'

export interface ElmPluginOptions {
  debug?: boolean
  optimize?: boolean
  compiler?: {
    compile(targets: string[]): Promise<string>
  }
  nodeElmCompilerOptions?: {
    cwd?: string
    docs?: string
    debug?: boolean
    optimize?: boolean
    processOpts?: Record<string, string>
    report?: string
    pathToElm?: string
    verbose?: boolean
  }
}

export default function elmPlugin(options?: ElmPluginOptions): Plugin
