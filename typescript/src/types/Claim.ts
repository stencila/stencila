// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ClaimType } from './ClaimType';
import { CreativeWork } from './CreativeWork';

// A claim represents specific reviewable facts or statements.
export class Claim extends CreativeWork {
  type = "Claim";

  // Content of the claim, usually a single paragraph.
  content: Block[];

  // The type of the claim.
  claimType: ClaimType;

  // A short label for the claim.
  label?: string;

  constructor(content: Block[], claimType: ClaimType, options?: Claim) {
    super()
    if (options) Object.assign(this, options)
    this.content = content;
    this.claimType = claimType;
  }
}
