// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { SectionType } from "./SectionType.js";

/**
 * A section of a document.
 */
export class Section extends Entity {
  type = "Section";

  /**
   * The content within the section.
   */
  content: Block[];

  /**
   * The type of section.
   */
  sectionType?: SectionType;

  constructor(content: Block[], options?: Partial<Section>) {
    super();
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Section`
*/
export function section(content: Block[], options?: Partial<Section>): Section {
  return new Section(content, options);
}
