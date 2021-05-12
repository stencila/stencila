// Node.js bindings for ../../rust/src/projects.rs, see there for more documentation.

import { JSONSchema7 } from 'json-schema'
import { fromJSON } from './prelude'

const addon = require('../index.node')

// Warning: The following types are hand written and may become out of sync
// with the actual JSON data returned by the functions below.
// Use the `schema()` function as the authoritative source of the shape of
// the project objects.

export interface Project {
  // Properties from the project's manifest file
  // See Rust docs and help.stenci.la for descriptions of these

  name: string
  description?: string
  image?: string
  main?: string
  theme?: string
}

/**
 * Get the JSON schema for a project object
 *
 * @returns A JSON Schema v7 object describing the properties of
 *          a project object
 */
export function schema(): JSONSchema7 {
  return fromJSON<JSONSchema7>(addon.projectsSchema())
}

/**
 * List projects that are currently open
 *
 * @returns An array of projects
 */
export function list(): Project[] {
  return fromJSON<Project[]>(addon.projectsList())
}

/**
 * Open a project
 *
 * @param path Path to the project folder
 * @return A project
 */
export function open(
  path: string
): Project {
  return fromJSON<Project>(addon.projectsOpen(path))
}

/**
 * Close a project
 *
 * @param Path to the project folder
 */
export function uninstall(path: string): void {
  addon.projectsClose(path)
}
