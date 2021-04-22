// Node.js bindings for ../../rust/src/plugins.rs, see there for more documentation.

import { fromJSON } from './prelude'

const addon = require('../index.node')

export type Installation = 'docker' | 'binary' | 'js' | 'py' | 'r' | 'link'

export interface Plugin {
  // Properties from the plugin's manifest file

  name: string
  softwareVersion: string
  description: string
  installUrl: string[]
  featureList: Record<string, unknown>[]

  // Properties that are derived / updated

  installation?: Installation
  refreshed?: string
  next?: Plugin
  alias?: string
}

/**
 * List plugins in registry and/or installed
 *
 * Consider using `plugins.refresh` instead to get a list of plugins that
 * has been updated with latest available version etc.
 *
 * @returns An array of plugins
 */
export function list(): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsList())
}

/**
 * Install a plugin
 *
 * @param spec A plugin identifier or spec e.g. `javascript`, `stencila/jesta@0.5.1`
 * @param installations An array of installation methods to try
 * @return An array of plugins
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
 * @returns An array of plugins
 */
export function uninstall(alias: string): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsUninstall(alias))
}

/**
 * Upgrade a plugin
 *
 * @param spec A plugin identifier or spec e.g. `javascript`
 * @return An array of plugins
 */
export function upgrade(spec: string): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsUpgrade(spec))
}

/**
 * Refresh plugins
 *
 * @param list A list of plugin aliases or names to refresh.
 *             Use an empty array to refresh all plugins
 * @return An array of plugins
 */
export function refresh(list: string[]): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsRefresh(list))
}
