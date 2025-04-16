// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Block } from "./Block.js";
import { CitationOrText } from "./CitationOrText.js";
import { Entity } from "./Entity.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

/**
 * A section quoted from somewhere else.
 */
export class QuoteBlock extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "QuoteBlock";

  /**
   * The source of the quote.
   */
  source?: CitationOrText;

  /**
   * The content of the quote.
   */
  content: Block[];

  /**
   * The authors of the quote.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the content within the section.
   */
  provenance?: ProvenanceCount[];

  constructor(content: Block[], options?: Partial<QuoteBlock>) {
    super();
    this.type = "QuoteBlock";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `QuoteBlock`
*/
export function quoteBlock(content: Block[], options?: Partial<QuoteBlock>): QuoteBlock {
  return new QuoteBlock(content, options);
}
