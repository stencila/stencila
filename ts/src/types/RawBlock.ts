// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

/**
 * Document content in a specific format
 */
export class RawBlock extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "RawBlock";

  /**
   * The format of the raw content.
   */
  format: string;

  /**
   * The raw content.
   */
  content: Cord;

  /**
   * A digest of the `format` and `content` properties.
   */
  compilationDigest?: CompilationDigest;

  /**
   * Messages generated while parsing and transpiling the `content` into the `css` property.
   */
  compilationMessages?: CompilationMessage[];

  /**
   * A Cascading Style Sheet (CSS) generated from the `content`.
   */
  css?: string;

  /**
   * The authors of the content.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the content.
   */
  provenance?: ProvenanceCount[];

  constructor(format: string, content: Cord, options?: Partial<RawBlock>) {
    super();
    this.type = "RawBlock";
    if (options) Object.assign(this, options);
    this.format = format;
    this.content = content;
  }
}

/**
* Create a new `RawBlock`
*/
export function rawBlock(format: string, content: Cord, options?: Partial<RawBlock>): RawBlock {
  return new RawBlock(format, content, options);
}
