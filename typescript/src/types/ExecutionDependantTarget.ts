// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Call } from "./Call.js";
import { CodeChunk } from "./CodeChunk.js";
import { CodeExpression } from "./CodeExpression.js";
import { Division } from "./Division.js";
import { File } from "./File.js";
import { For } from "./For.js";
import { If } from "./If.js";
import { Span } from "./Span.js";
import { Variable } from "./Variable.js";

// Node types that can be execution dependants
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

export function executionDependantTargetFrom(other: ExecutionDependantTarget): ExecutionDependantTarget {
  switch(other.type) {
    case "Call": return Call.from(other as Call);
    case "CodeChunk": return CodeChunk.from(other as CodeChunk);
    case "CodeExpression": return CodeExpression.from(other as CodeExpression);
    case "Division": return Division.from(other as Division);
    case "If": return If.from(other as If);
    case "File": return File.from(other as File);
    case "For": return For.from(other as For);
    case "Span": return Span.from(other as Span);
    case "Variable": return Variable.from(other as Variable);
    default: throw new Error(`Unexpected type for ExecutionDependantTarget: ${other.type}`);
  }
}
