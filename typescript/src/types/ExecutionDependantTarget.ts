// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { Call } from "./Call.js";
import { CodeChunk } from "./CodeChunk.js";
import { CodeExpression } from "./CodeExpression.js";
import { Division } from "./Division.js";
import { File } from "./File.js";
import { For } from "./For.js";
import { If } from "./If.js";
import { Span } from "./Span.js";
import { Variable } from "./Variable.js";

/**
 * Node types that can be execution dependants
 */
export type ExecutionDependantTarget =
  Call |
  CodeChunk |
  CodeExpression |
  Division |
  If |
  File |
  For |
  Span |
  Variable;

/**
 * Create a `ExecutionDependantTarget` from an object
 */
export function executionDependantTarget(other: ExecutionDependantTarget): ExecutionDependantTarget {
  switch(other.type) {
    case "Call":
    case "CodeChunk":
    case "CodeExpression":
    case "Division":
    case "If":
    case "File":
    case "For":
    case "Span":
    case "Variable":
      return hydrate(other) as ExecutionDependantTarget
    default:
      throw new Error(`Unexpected type for ExecutionDependantTarget: ${other.type}`);
  }
}
