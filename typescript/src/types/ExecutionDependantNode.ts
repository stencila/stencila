// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Button } from './Button'
import { Call } from './Call'
import { CodeChunk } from './CodeChunk'
import { CodeExpression } from './CodeExpression'
import { Division } from './Division'
import { File } from './File'
import { Parameter } from './Parameter'
import { Span } from './Span'
import { Variable } from './Variable'

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

export function executionDependantNode(other: ExecutionDependantNode): ExecutionDependantNode {
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
    default: throw new Error(`Unexpected type for ExecutionDependantNode: ${other.type}`)
  }
}
