// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Parameter } from "./Parameter.js";

/**
 * The value of a `Parameter` to call a document with.
 */
export class CallArgument extends Parameter {
  type = "CallArgument";

  /**
   * The code to be evaluated for the parameter.
   */
  code: string;

  /**
   * The programming language of the code.
   */
  programmingLanguage?: string;

  constructor(name: string, code: string, options?: Partial<CallArgument>) {
    super(name);
    if (options) Object.assign(this, options);
    this.name = name;
    this.code = code;
  }
}

/**
* Create a new `CallArgument`
*/
export function callArgument(name: string, code: string, options?: Partial<CallArgument>): CallArgument {
  return new CallArgument(name, code, options);
}
