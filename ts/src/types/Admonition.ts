// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { AdmonitionType } from "./AdmonitionType.js";
import { Author } from "./Author.js";
import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

/**
 * A admonition within a document.
 */
export class Admonition extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Admonition";

  /**
   * The type of admonition.
   */
  admonitionType: AdmonitionType;

  /**
   * The title of the admonition.
   */
  title?: Inline[];

  /**
   * Whether the admonition is folded.
   */
  isFolded?: boolean;

  /**
   * The content within the section.
   */
  content: Block[];

  /**
   * The authors of the admonition.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the content within the admonition.
   */
  provenance?: ProvenanceCount[];

  constructor(admonitionType: AdmonitionType, content: Block[], options?: Partial<Admonition>) {
    super();
    this.type = "Admonition";
    if (options) Object.assign(this, options);
    this.admonitionType = admonitionType;
    this.content = content;
  }
}

/**
* Create a new `Admonition`
*/
export function admonition(admonitionType: AdmonitionType, content: Block[], options?: Partial<Admonition>): Admonition {
  return new Admonition(admonitionType, content, options);
}
