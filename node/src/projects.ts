// Node.js bindings for ../../rust/src/projects.rs, see there for more documentation.

import { JSONSchema7 } from 'json-schema'
import { fromJSON } from './prelude'
import { subscribe, Subscriber } from './pubsub'
import { Project } from './types'

const addon = require('../index.node')

/**
 * Get the JSON schema for a project object
 *
 * @returns A JSON Schema v7 object describing the properties of
 *          a project object
 */
export function schemas(): JSONSchema7[] {
  return fromJSON<JSONSchema7[]>(addon.projectsSchemas())
}

/**
 * List projects that are currently open
 *
 * @returns An array of project paths (relative to the current working directory)
 */
export function list(): string[] {
  return fromJSON<string[]>(addon.projectsList())
}

/**
 * Open a project
 *
 * @param path Path to the project folder
 * @param projectEventsSubscriber A subscriber function that will receive `ProjectEvent`s
 * @param fileEventsSubscriber A subscriber function that will receive `FileEvent`s
 * @return A project
 */
export function open(
  folder: string,
  projectEventsSubscriber?: Subscriber,
  fileEventsSubscriber?: Subscriber
): Project {
  const project = fromJSON<Project>(addon.projectsOpen(folder))
  if (projectEventsSubscriber !== undefined)
    subscribe(`projects:${project.path}:props`, projectEventsSubscriber)
  if (fileEventsSubscriber !== undefined)
    subscribe(`projects:${project.path}:files`, fileEventsSubscriber)
  return project as Project
}

/**
 * Write a project's `project.json` file
 * 
 * TODO: Allow passing of new project properties
 * to the project.
 *
 * @param path Path to the project folder
 * @return A project
 */
export function write(folder: string) {
  addon.projectsWrite(folder)
}

/**
 * Close a project
 *
 * @param Path to the project folder
 */
export function close(folder: string): void {
  addon.projectsClose(folder)
}
