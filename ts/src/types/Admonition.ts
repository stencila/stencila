// Generated file; do not edit. See `../rust/schema-gen` crate.

import { AdmonitionType } from "./AdmonitionType.js";
import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";

/**
 * A admonition within a document.
 */
export class Admonition extends Entity {
  type = "Admonition";

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

  constructor(admonitionType: AdmonitionType, content: Block[], options?: Partial<Admonition>) {
    super();
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
