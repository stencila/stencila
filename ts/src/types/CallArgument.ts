// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Cord } from "./Cord.js";
import { Node } from "./Node.js";
import { Parameter } from "./Parameter.js";

/**
 * The value of a `Parameter` to call a document with.
 */
export class CallArgument extends Parameter {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CallArgument";

  /**
   * The current value of the argument.
   */
  value?: Node;

  /**
   * The code to be evaluated for the parameter.
   */
  code: Cord;

  /**
   * The programming language of the code.
   */
  programmingLanguage?: string;

  constructor(name: string, code: Cord, options?: Partial<CallArgument>) {
    super(name);
    this.type = "CallArgument";
    if (options) Object.assign(this, options);
    this.name = name;
    this.code = code;
  }
}

/**
* Create a new `CallArgument`
*/
export function callArgument(name: string, code: Cord, options?: Partial<CallArgument>): CallArgument {
  return new CallArgument(name, code, options);
}
