// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { ClaimType } from "./ClaimType.js";
import { CreativeWork } from "./CreativeWork.js";

/**
 * A claim represents specific reviewable facts or statements.
 */
export class Claim extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Claim";

  /**
   * The type of the claim.
   */
  claimType: ClaimType;

  /**
   * A short label for the claim.
   */
  label?: string;

  /**
   * Content of the claim, usually a single paragraph.
   */
  content: Block[];

  constructor(claimType: ClaimType, content: Block[], options?: Partial<Claim>) {
    super();
    this.type = "Claim";
    if (options) Object.assign(this, options);
    this.claimType = claimType;
    this.content = content;
  }
}

/**
* Create a new `Claim`
*/
export function claim(claimType: ClaimType, content: Block[], options?: Partial<Claim>): Claim {
  return new Claim(claimType, content, options);
}
