// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Call } from "./Call.js";
import { type CodeChunk } from "./CodeChunk.js";
import { type CodeExpression } from "./CodeExpression.js";
import { type Division } from "./Division.js";
import { type File } from "./File.js";
import { type For } from "./For.js";
import { type If } from "./If.js";
import { type Span } from "./Span.js";
import { type Variable } from "./Variable.js";

/**
 * Node types that can be execution dependants.
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
