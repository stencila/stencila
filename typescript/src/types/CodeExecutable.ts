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
  programmingLanguage: string;

  /**
   * Whether the programming language of the code should be guessed based on syntax and variables used
   */
  guessLanguage?: boolean;

  constructor(code: Cord, programmingLanguage: string, options?: Partial<CodeExecutable>) {
    super();
    if (options) Object.assign(this, options);
    this.code = code;
    this.programmingLanguage = programmingLanguage;
  }

  /**
  * Create a `CodeExecutable` from an object
  */
  static from(other: CodeExecutable): CodeExecutable {
    return new CodeExecutable(other.code!, other.programmingLanguage!, other);
  }
}
