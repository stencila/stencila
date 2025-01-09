// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Annotated content.
 */
export class Annotation extends Mark {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Annotation";

  /**
   * The annotation, usually a paragraph.
   */
  annotation?: Block[];

  constructor(content: Inline[], options?: Partial<Annotation>) {
    super(content);
    this.type = "Annotation";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Annotation`
*/
export function annotation(content: Inline[], options?: Partial<Annotation>): Annotation {
  return new Annotation(content, options);
}
