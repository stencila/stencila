// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { ProvenanceCategory } from "./ProvenanceCategory.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * The count of the number of characters in a `ProvenanceCategory` within an entity.
 */
export class ProvenanceCount extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ProvenanceCount";

  /**
   * The provenance category that the character count applies to.
   */
  provenanceCategory: ProvenanceCategory;

  /**
   * The number of characters in the provenance category.
   */
  characterCount: UnsignedInteger;

  /**
   * The percentage of characters in the provenance category.
   */
  characterPercent?: UnsignedInteger;

  constructor(provenanceCategory: ProvenanceCategory, characterCount: UnsignedInteger, options?: Partial<ProvenanceCount>) {
    super();
    this.type = "ProvenanceCount";
    if (options) Object.assign(this, options);
    this.provenanceCategory = provenanceCategory;
    this.characterCount = characterCount;
  }
}

/**
* Create a new `ProvenanceCount`
*/
export function provenanceCount(provenanceCategory: ProvenanceCategory, characterCount: UnsignedInteger, options?: Partial<ProvenanceCount>): ProvenanceCount {
  return new ProvenanceCount(provenanceCategory, characterCount, options);
}
