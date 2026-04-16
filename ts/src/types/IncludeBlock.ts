// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Executable } from "./Executable.js";

/**
 * A block that includes content from an external source.
 */
export class IncludeBlock extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "IncludeBlock";

  /**
   * The external source of the content, a file path or URL.
   */
  source: string;

  /**
   * Media type of the source content.
   */
  mediaType?: string;

  /**
   * A query to select a subset of content from the source
   */
  select?: string;

  /**
   * The structured content decoded from the source.
   */
  content?: Block[];

  constructor(source: string, options?: Partial<IncludeBlock>) {
    super();
    this.type = "IncludeBlock";
    if (options) Object.assign(this, options);
    this.source = source;
  }
}

/**
* Create a new `IncludeBlock`
*/
export function includeBlock(source: string, options?: Partial<IncludeBlock>): IncludeBlock {
  return new IncludeBlock(source, options);
}
