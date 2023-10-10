// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { ClaimType } from "./ClaimType.js";
import { CreativeWork } from "./CreativeWork.js";

/**
 * A claim represents specific reviewable facts or statements.
 */
export class Claim extends CreativeWork {
  type = "Claim";

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
