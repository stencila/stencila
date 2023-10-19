// Generated file; do not edit. See `../rust/schema-gen` crate.

import { BlocksOrInlines } from "./BlocksOrInlines.js";
import { CodeExecutable } from "./CodeExecutable.js";
import { Cord } from "./Cord.js";
import { Node } from "./Node.js";

/**
 * A executable chunk of code.
 */
export class CodeChunk extends CodeExecutable {
  type = "CodeChunk";

  /**
   * Whether the code should be treated as side-effect free when executed.
   */
  executionPure?: boolean;

  /**
   * Outputs from executing the chunk.
   */
  outputs?: Node[];

  /**
   * A short label for the CodeChunk.
   */
  label?: string;

  /**
   * A caption for the CodeChunk.
   */
  caption?: BlocksOrInlines;

  constructor(code: Cord, programmingLanguage: string, options?: Partial<CodeChunk>) {
    super(code, programmingLanguage);
    if (options) Object.assign(this, options);
    this.code = code;
    this.programmingLanguage = programmingLanguage;
  }
}

/**
* Create a new `CodeChunk`
*/
export function codeChunk(code: Cord, programmingLanguage: string, options?: Partial<CodeChunk>): CodeChunk {
  return new CodeChunk(code, programmingLanguage, options);
}
