// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";
import { MessageLevel } from "./MessageLevel.js";

/**
 * An error, warning or log message generated during execution.
 */
export class ExecutionMessage extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ExecutionMessage";

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
   * The location that the error occurred or other message emanated from.
   */
  codeLocation?: CodeLocation;

  /**
   * Stack trace leading up to the error.
   */
  stackTrace?: string;

  constructor(level: MessageLevel, message: string, options?: Partial<ExecutionMessage>) {
    super();
    this.type = "ExecutionMessage";
    if (options) Object.assign(this, options);
    this.level = level;
    this.message = message;
  }
}

/**
* Create a new `ExecutionMessage`
*/
export function executionMessage(level: MessageLevel, message: string, options?: Partial<ExecutionMessage>): ExecutionMessage {
  return new ExecutionMessage(level, message, options);
}
