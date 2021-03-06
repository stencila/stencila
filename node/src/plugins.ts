// Node.js bindings for ../../rust/src/plugins.rs, see there for more documentation.

import { JSONSchema7 } from 'json-schema'
import { fromJSON } from './prelude'
import { Plugin } from './types'

const addon = require('../index.node')

/**
 * Get the JSON schema for a plugin object
 *
 * @returns A JSON Schema v7 object describing the properties of
 *          a plugin object
 */
export function schema(): JSONSchema7 {
  return fromJSON<JSONSchema7>(addon.pluginsSchema())
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
  installations?: Plugin['installation'] | Plugin['installation'][]
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
 * Refresh the metadata for one or more plugins
 *
 * This does not upgrade installed plugins. It fetches the
 * latest manifest for the plugin, which if it is already
 * installed will be installed in the `next` property.
 *
 * @param list A list of plugin aliases or names to refresh.
 *             Use an empty array to refresh all plugins
 * @return An array of plugins
 */
export function refresh(list: string[]): Plugin[] {
  return fromJSON<Plugin[]>(addon.pluginsRefresh(list))
}
