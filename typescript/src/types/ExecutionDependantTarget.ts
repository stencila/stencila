// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Call } from './Call'
import { CodeChunk } from './CodeChunk'
import { CodeExpression } from './CodeExpression'
import { Division } from './Division'
import { File } from './File'
import { For } from './For'
import { If } from './If'
import { Span } from './Span'
import { Variable } from './Variable'

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

export function executionDependantTarget(other: ExecutionDependantTarget): ExecutionDependantTarget {
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
    default: throw new Error(`Unexpected type for ExecutionDependantTarget: ${other.type}`)
  }
}
