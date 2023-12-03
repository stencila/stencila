// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from "./Cord.js";
import { Executable } from "./Executable.js";

/**
 * Abstract base type for executable code nodes (e.g. `CodeChunk`).
 */
export class CodeExecutable extends Executable {
  type = "CodeExecutable";

  /**
   * The code.
   */
  code: Cord;

  /**
   * The programming language of the code.
   */
  programmingLanguage?: string;

  constructor(code: Cord, options?: Partial<CodeExecutable>) {
    super();
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeExecutable`
*/
export function codeExecutable(code: Cord, options?: Partial<CodeExecutable>): CodeExecutable {
  return new CodeExecutable(code, options);
}
