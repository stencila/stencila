// Generated file; do not edit. See `../rust/schema-gen` crate.

import { BlocksOrString } from './BlocksOrString';
import { CodeExecutable } from './CodeExecutable';
import { Cord } from './Cord';
import { Node } from './Node';

// A executable chunk of code.
export class CodeChunk extends CodeExecutable {
  type = "CodeChunk";

  // Whether the code should be treated as side-effect free when executed.
  executionPure?: boolean;

  // Outputs from executing the chunk.
  outputs?: Node[];

  // A short label for the CodeChunk.
  label?: string;

  // A caption for the CodeChunk.
  caption?: BlocksOrString;

  constructor(code: Cord, programmingLanguage: string, options?: CodeChunk) {
    super(code, programmingLanguage)
    if (options) Object.assign(this, options)
    this.code = code;
    this.programmingLanguage = programmingLanguage;
  }
}
