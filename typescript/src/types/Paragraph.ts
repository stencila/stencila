// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";

/**
 * Paragraph
 */
export class Paragraph extends Entity {
  type = "Paragraph";

  /**
   * The contents of the paragraph.
   */
  content: Inline[];

  constructor(content: Inline[], options?: Partial<Paragraph>) {
    super();
    if (options) Object.assign(this, options);
    this.content = content;
  }

  /**
  * Create a `Paragraph` from an object
  */
  static from(other: Paragraph): Paragraph {
    return new Paragraph(other.content!, other);
  }
}

/**
* Create a new `Paragraph`
*/
export function paragraph(content: Inline[], options?: Partial<Paragraph>): Paragraph {
  return new Paragraph(content, options);
}
