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
