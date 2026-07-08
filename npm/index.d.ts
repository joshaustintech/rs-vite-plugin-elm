import type { Plugin } from 'vite'

export interface ElmPluginOptions {
  debug?: boolean
  optimize?: boolean
  nodeElmCompilerOptions?: {
    pathToElm?: string
  }
}

export default function elmPlugin(options?: ElmPluginOptions): Plugin
