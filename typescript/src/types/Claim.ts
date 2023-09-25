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
   * Content of the claim, usually a single paragraph.
   */
  content: Block[];

  /**
   * The type of the claim.
   */
  claimType: ClaimType;

  /**
   * A short label for the claim.
   */
  label?: string;

  constructor(content: Block[], claimType: ClaimType, options?: Partial<Claim>) {
    super();
    if (options) Object.assign(this, options);
    this.content = content;
    this.claimType = claimType;
  }

  /**
  * Create a `Claim` from an object
  */
  static from(other: Claim): Claim {
    return new Claim(other.content!, other.claimType!, other);
  }
}
