// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Button } from "./Button.js";
import { Call } from "./Call.js";
import { CodeChunk } from "./CodeChunk.js";
import { CodeExpression } from "./CodeExpression.js";
import { Division } from "./Division.js";
import { File } from "./File.js";
import { Parameter } from "./Parameter.js";
import { Span } from "./Span.js";
import { Variable } from "./Variable.js";

// Node types that can be execution dependencies
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

export function executionDependantNodeFrom(other: ExecutionDependantNode): ExecutionDependantNode {
  switch(other.type) {
    case "Button": return Button.from(other as Button);
    case "Call": return Call.from(other as Call);
    case "CodeChunk": return CodeChunk.from(other as CodeChunk);
    case "CodeExpression": return CodeExpression.from(other as CodeExpression);
    case "Division": return Division.from(other as Division);
    case "File": return File.from(other as File);
    case "Parameter": return Parameter.from(other as Parameter);
    case "Span": return Span.from(other as Span);
    case "Variable": return Variable.from(other as Variable);
    default: throw new Error(`Unexpected type for ExecutionDependantNode: ${other.type}`);
  }
}
