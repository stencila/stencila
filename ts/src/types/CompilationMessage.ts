// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";
import { MessageLevel } from "./MessageLevel.js";

/**
 * An error, warning or log message generated during compilation.
 */
export class CompilationMessage extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CompilationMessage";

  /**
   * The severity level of the message.
   */
  level: MessageLevel;

  /**
   * The text of the message.
   */
  message: string;

  /**
   * The type of error e.g. "SyntaxError", "ZeroDivisionError".
   */
  errorType?: string;

  /**
   * The location that the error occurred.
   */
  codeLocation?: CodeLocation;

  constructor(level: MessageLevel, message: string, options?: Partial<CompilationMessage>) {
    super();
    this.type = "CompilationMessage";
    if (options) Object.assign(this, options);
    this.level = level;
    this.message = message;
  }
}

/**
* Create a new `CompilationMessage`
*/
export function compilationMessage(level: MessageLevel, message: string, options?: Partial<CompilationMessage>): CompilationMessage {
  return new CompilationMessage(level, message, options);
}
