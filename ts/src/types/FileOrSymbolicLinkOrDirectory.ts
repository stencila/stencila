// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Directory } from "./Directory.js";
import { type File } from "./File.js";
import { type SymbolicLink } from "./SymbolicLink.js";

/**
 * `File` or `SymbolicLink` or `Directory`
 */
export type FileOrSymbolicLinkOrDirectory =
  File |
  SymbolicLink |
  Directory;

/**
 * Create a `FileOrSymbolicLinkOrDirectory` from an object
 */
export function fileOrSymbolicLinkOrDirectory(other: FileOrSymbolicLinkOrDirectory): FileOrSymbolicLinkOrDirectory {
  switch(other.type) {
    case "File":
    case "SymbolicLink":
    case "Directory":
      return hydrate(other) as FileOrSymbolicLinkOrDirectory
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for FileOrSymbolicLinkOrDirectory: ${other.type}`);
  }
}
