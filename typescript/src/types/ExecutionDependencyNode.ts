// Generated file; do not edit. See `../rust/schema-gen` crate.
            
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
    case "Button": return Button.from(other as Button);
    case "CodeChunk": return CodeChunk.from(other as CodeChunk);
    case "File": return File.from(other as File);
    case "Parameter": return Parameter.from(other as Parameter);
    case "SoftwareSourceCode": return SoftwareSourceCode.from(other as SoftwareSourceCode);
    case "Variable": return Variable.from(other as Variable);
    default: throw new Error(`Unexpected type for ExecutionDependencyNode: ${other.type}`);
  }
}
