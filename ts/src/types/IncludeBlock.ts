// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Executable } from "./Executable.js";

/**
 * Include block content from an external source (e.g. file, URL).
 */
export class IncludeBlock extends Executable {
  type = "IncludeBlock";

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
