// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

/**
 * A paragraph.
 */
export class Paragraph extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Paragraph";

  /**
   * The contents of the paragraph.
   */
  content: Inline[];

  /**
   * The authors of the paragraph.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of content within the paragraph.
   */
  provenance?: ProvenanceCount[];

  constructor(content: Inline[], options?: Partial<Paragraph>) {
    super();
    this.type = "Paragraph";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Paragraph`
*/
export function paragraph(content: Inline[], options?: Partial<Paragraph>): Paragraph {
  return new Paragraph(content, options);
}
