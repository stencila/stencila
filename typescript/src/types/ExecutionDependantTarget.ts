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
