// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { ClaimType } from "./ClaimType.js";
import { ResearchObject } from "./ResearchObject.js";

/**
 * A reviewable claim or statement.
 */
export class Claim extends ResearchObject {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Claim";

  /**
   * The type of the claim.
   */
  claimType?: ClaimType;

  constructor(content: Block[], options?: Partial<Claim>) {
    super(content);
    this.type = "Claim";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Claim`
*/
export function claim(content: Block[], options?: Partial<Claim>): Claim {
  return new Claim(content, options);
}
