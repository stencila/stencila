// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { ResearchObject } from "./ResearchObject.js";

/**
 * Evidence supporting, opposing, or otherwise informing a research claim.
 */
export class Evidence extends ResearchObject {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Evidence";

  constructor(content: Block[], options?: Partial<Evidence>) {
    super(content);
    this.type = "Evidence";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Evidence`
*/
export function evidence(content: Block[], options?: Partial<Evidence>): Evidence {
  return new Evidence(content, options);
}
