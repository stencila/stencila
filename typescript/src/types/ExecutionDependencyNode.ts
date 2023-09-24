// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Button } from './Button'
import { CodeChunk } from './CodeChunk'
import { File } from './File'
import { Parameter } from './Parameter'
import { SoftwareSourceCode } from './SoftwareSourceCode'
import { Variable } from './Variable'

// Node types that can be execution dependencies
export type ExecutionDependencyNode =
  Button |
  CodeChunk |
  File |
  Parameter |
  SoftwareSourceCode |
  Variable;

export function executionDependencyNode(other: ExecutionDependencyNode): ExecutionDependencyNode {
  switch(other.type) {
    case "Button": return Button.from(other as Button);
    case "CodeChunk": return CodeChunk.from(other as CodeChunk);
    case "File": return File.from(other as File);
    case "Parameter": return Parameter.from(other as Parameter);
    case "SoftwareSourceCode": return SoftwareSourceCode.from(other as SoftwareSourceCode);
    case "Variable": return Variable.from(other as Variable);
    default: throw new Error(`Unexpected type for ExecutionDependencyNode: ${other.type}`)
  }
}
