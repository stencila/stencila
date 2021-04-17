import { fromJSON } from './prelude'

const addon = require('../native')

export type Installation = 'binary' | 'docker' | 'package'

export interface Plugin {
  // Properties from the plugin's codemeta.json file
  name: string
  softwareVersion: string
  description: string
  installUrl: string[]
  featureList: Record<string, unknown>[]

  // If installed, the installation type
  installation?: Installation

  // The current alias for this plugin, if any
  alias?: string
}

/**
 * List the installed plugins
 *
 * @returns An array of plugins
 */
export function list(): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsList())
}

/**
 * Install a plugin
 *
 * @param spec A plugin identifier e.g. `javascript@0.50.1`
 * @param installations An array of installation methods to try
 * @return An array of installed plugins
 */
export function install(
  spec: string,
  installations?: Installation | Installation[]
): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsInstall(spec, installations ?? []))
}

/**
 * Uninstall a plugin
 *
 * @param alias The alias or name of the plugin
 * @returns An array of installed plugins
 */
export function uninstall(alias: string): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsUninstall(alias))
}

/**
 * Upgrade a plugin
 *
 * @param spec A plugin identifier e.g. `javascript`
 * @return An array of installed plugins
 */
export function upgrade(spec: string): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsUpgrade(spec))
}
