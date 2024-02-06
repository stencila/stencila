// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Directory } from "./Directory.js";
import { type File } from "./File.js";

/**
 * `File` or `Directory`
 */
export type FileOrDirectory =
  File |
  Directory;

/**
 * Create a `FileOrDirectory` from an object
 */
export function fileOrDirectory(other: FileOrDirectory): FileOrDirectory {
  switch(other.type) {
    case "File":
    case "Directory":
      return hydrate(other) as FileOrDirectory
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for FileOrDirectory: ${other.type}`);
  }
}
