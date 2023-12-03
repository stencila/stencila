// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Button } from "./Button.js";
import { type CodeChunk } from "./CodeChunk.js";
import { type File } from "./File.js";
import { type Parameter } from "./Parameter.js";
import { type SoftwareSourceCode } from "./SoftwareSourceCode.js";
import { type Variable } from "./Variable.js";

/**
 * Node types that can be execution dependencies.
 */
export type ExecutionDependencyNode =
  Button |
  CodeChunk |
  File |
  Parameter |
  SoftwareSourceCode |
  Variable;

/**
 * Create a `ExecutionDependencyNode` from an object
 */
export function executionDependencyNode(other: ExecutionDependencyNode): ExecutionDependencyNode {
  switch(other.type) {
    case "Button":
    case "CodeChunk":
    case "File":
    case "Parameter":
    case "SoftwareSourceCode":
    case "Variable":
      return hydrate(other) as ExecutionDependencyNode
    default:
      throw new Error(`Unexpected type for ExecutionDependencyNode: ${other.type}`);
  }
}
