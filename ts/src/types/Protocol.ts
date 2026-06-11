// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { ResearchObject } from "./ResearchObject.js";

/**
 * A research protocol or method description.
 */
export class Protocol extends ResearchObject {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Protocol";

  constructor(content: Block[], options?: Partial<Protocol>) {
    super(content);
    this.type = "Protocol";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Protocol`
*/
export function protocol(content: Block[], options?: Partial<Protocol>): Protocol {
  return new Protocol(content, options);
}
