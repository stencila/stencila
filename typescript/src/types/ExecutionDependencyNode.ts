// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { Button } from "./Button.js";
import { CodeChunk } from "./CodeChunk.js";
import { File } from "./File.js";
import { Parameter } from "./Parameter.js";
import { SoftwareSourceCode } from "./SoftwareSourceCode.js";
import { Variable } from "./Variable.js";

/**
 * Node types that can be execution dependencies
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
