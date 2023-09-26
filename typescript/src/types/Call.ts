// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CallArgument } from "./CallArgument.js";
import { Include } from "./Include.js";

/**
 * Call another document, optionally with arguments, and include its executed content.
 */
export class Call extends Include {
  type = "Call";

  /**
   * The value of the source document's parameters to call it with
   */
  arguments: CallArgument[];

  constructor(source: string, args: CallArgument[], options?: Partial<Call>) {
    super(source);
    if (options) Object.assign(this, options);
    this.source = source;
    this.arguments = args;
  }
}

/**
* Create a new `Call`
*/
export function call(source: string, args: CallArgument[], options?: Partial<Call>): Call {
  return new Call(source, args, options);
}
