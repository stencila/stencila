// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { SectionType } from "./SectionType.js";

/**
 * A section of a document.
 */
export class Section extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Section";

  /**
   * The content within the section.
   */
  content: Block[];

  /**
   * The type of section.
   */
  sectionType?: SectionType;

  /**
   * The authors of the section.
   */
  authors?: Author[];

  constructor(content: Block[], options?: Partial<Section>) {
    super();
    this.type = "Section";
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
