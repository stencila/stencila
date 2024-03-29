// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CallArgument } from "./CallArgument.js";
import { IncludeBlock } from "./IncludeBlock.js";

/**
 * Call another document, optionally with arguments, and include its executed content.
 */
export class CallBlock extends IncludeBlock {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CallBlock";

  /**
   * The value of the source document's parameters to call it with
   */
  arguments: CallArgument[];

  constructor(source: string, args: CallArgument[], options?: Partial<CallBlock>) {
    super(source);
    this.type = "CallBlock";
    if (options) Object.assign(this, options);
    this.source = source;
    this.arguments = args;
  }
}

/**
* Create a new `CallBlock`
*/
export function callBlock(source: string, args: CallArgument[], options?: Partial<CallBlock>): CallBlock {
  return new CallBlock(source, args, options);
}
