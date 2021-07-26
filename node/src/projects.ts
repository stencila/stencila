// Node.js bindings for ../../rust/src/projects.rs, see there for more documentation.

import { JSONSchema7 } from 'json-schema'
import { fromJSON, toJSON } from './prelude'
import * as pubsub from './pubsub'
import { FileEvent, Project, ProjectEvent } from './types'

const addon = require('../index.node')

/**
 * Get the JSON Schemas associated with the `projects` module.
 *
 * @returns An array of JSON Schema v7 objects
 */
export function schemas(): JSONSchema7[] {
  return fromJSON<JSONSchema7[]>(addon.projectsSchemas())
}

/**
 * List projects that are currently open.
 *
 * @returns An array of project paths (relative to the current working directory)
 */
export function list(): string[] {
  return fromJSON<string[]>(addon.projectsList())
}

/**
 * Open an existing project.
 *
 * @param path Path to the project folder
 * @return A `Project` object
 */
export function open(path: string): Project {
  return fromJSON<Project>(addon.projectsOpen(path))
}

/**
 * Close a project.
 *
 * This will drop the project from memory and stop any
 * associated file watching thread.
 *
 * @param path Path to the project folder
 */
export function close(path: string): void {
  addon.projectsClose(path)
}

/**
 * Write a project's `project.json` file with new settings.
 *
 * Note that some of the properties of a project are derived
 * and attempting to set these will be ignores. These readonly
 * properties include `files`, `mainPath` (set `main` instead),
 * and `imagePath` (set `image` instead).
 *
 * @param path Path to the project folder
 * @param updates A `Project` object with the settings to be updated.
 */
export function write(path: string, updates: Project) {
  addon.projectsWrite(path, toJSON(updates))
}

/**
 * Add a source to a project
 *
 * @param path Path to the project folder
 * @param source A URL or identifier for the source e.g. github:stencila/test
 * @param destination The destination for the source within the project folder
 * @param name The name for the source
 */
export function addSource(
  path: string,
  source: string,
  destination?: string,
  name?: string
) {
  addon.projectsAddSource(path, source, destination ?? '', name ?? '')
}

/**
 * Remove a source from a project
 *
 * @param name The name of the source
 */
 export function removeSource(
  path: string,
  name: string
) {
  addon.projectsRemoveSource(path, name)
}

/**
 * Import a new or existing source into a project
 *
 * @param path Path to the project folder
 * @param nameOrIdentifier Name of an existing source or an identifier for a new source
 * @param destination The destination for the source within the project folder
 */
 export function importSource(
  path: string,
  nameOrIdentifier: string,
  destination?: string
) {
  addon.projectsImportSource(path, nameOrIdentifier, destination ?? '')
}

/**
 * Subscribe to one or more of a project's topics.
 *
 * @param path Path to the project folder
 * @param topic See Rust docs for for valid values
 * @param subscriber A subscriber function that will receive published
 *                   events for the project topic/s
 */
export function subscribe(
  path: string,
  topics: string[],
  subscriber: (topic: string, event: ProjectEvent | FileEvent) => unknown
): void {
  for (const topic of topics) {
    pubsub.subscribe(
      `projects:${path}:${topic}`,
      subscriber as pubsub.Subscriber
    )
  }
}

/**
 * Unsubscribe from one or more of a project's topics.
 *
 * @param path Path to the project folder
 * @param subscriber A subscriber function that will receive published
 *                   events for the project topic/s
 */
export function unsubscribe(path: string, topics: string[]): void {
  for (const topic of topics) {
    pubsub.unsubscribe(`projects:${path}:${topic}`)
  }
}
