// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";
import { Integer } from "./Integer.js";
import { LabelType } from "./LabelType.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

/**
 * A heading.
 */
export class Heading extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Heading";

  /**
   * The type of the label for the appendix (if present, should be `AppendixLabel`).
   */
  labelType?: LabelType;

  /**
   * A short label for the heading.
   */
  label?: string;

  /**
   * The level of the heading.
   */
  level: Integer = 0;

  /**
   * Content of the heading.
   */
  content: Inline[];

  /**
   * The authors of the heading.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the content within the heading.
   */
  provenance?: ProvenanceCount[];

  constructor(level: Integer, content: Inline[], options?: Partial<Heading>) {
    super();
    this.type = "Heading";
    if (options) Object.assign(this, options);
    this.level = level;
    this.content = content;
  }
}

/**
* Create a new `Heading`
*/
export function heading(level: Integer, content: Inline[], options?: Partial<Heading>): Heading {
  return new Heading(level, content, options);
}
