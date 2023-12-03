// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Button } from "./Button.js";
import { type CallBlock } from "./CallBlock.js";
import { type CodeChunk } from "./CodeChunk.js";
import { type CodeExpression } from "./CodeExpression.js";
import { type File } from "./File.js";
import { type Function } from "./Function.js";
import { type Parameter } from "./Parameter.js";
import { type StyledBlock } from "./StyledBlock.js";
import { type StyledInline } from "./StyledInline.js";
import { type Variable } from "./Variable.js";

/**
 * Node types that can be execution dependencies.
 */
export type ExecutionDependantNode =
  Button |
  CallBlock |
  CodeChunk |
  CodeExpression |
  File |
  Function |
  Parameter |
  StyledBlock |
  StyledInline |
  Variable;

/**
 * Create a `ExecutionDependantNode` from an object
 */
export function executionDependantNode(other: ExecutionDependantNode): ExecutionDependantNode {
  switch(other.type) {
    case "Button":
    case "CallBlock":
    case "CodeChunk":
    case "CodeExpression":
    case "File":
    case "Function":
    case "Parameter":
    case "StyledBlock":
    case "StyledInline":
    case "Variable":
      return hydrate(other) as ExecutionDependantNode
    default:
      throw new Error(`Unexpected type for ExecutionDependantNode: ${other.type}`);
  }
}
