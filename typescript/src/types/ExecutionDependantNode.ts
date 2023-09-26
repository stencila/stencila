// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { Button } from "./Button.js";
import { Call } from "./Call.js";
import { CodeChunk } from "./CodeChunk.js";
import { CodeExpression } from "./CodeExpression.js";
import { Division } from "./Division.js";
import { File } from "./File.js";
import { Parameter } from "./Parameter.js";
import { Span } from "./Span.js";
import { Variable } from "./Variable.js";

/**
 * Node types that can be execution dependencies
 */
export type ExecutionDependantNode =
  Button |
  Call |
  CodeChunk |
  CodeExpression |
  Division |
  File |
  Parameter |
  Span |
  Variable;

/**
 * Create a `ExecutionDependantNode` from an object
 */
export function executionDependantNode(other: ExecutionDependantNode): ExecutionDependantNode {
  switch(other.type) {
    case "Button":
    case "Call":
    case "CodeChunk":
    case "CodeExpression":
    case "Division":
    case "File":
    case "Parameter":
    case "Span":
    case "Variable":
      return hydrate(other) as ExecutionDependantNode
    default:
      throw new Error(`Unexpected type for ExecutionDependantNode: ${other.type}`);
  }
}
