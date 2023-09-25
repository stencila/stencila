// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Executable } from "./Executable.js";

/**
 * Include content from an external source (e.g. file, URL).
 */
export class Include extends Executable {
  type = "Include";

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

  constructor(source: string, options?: Partial<Include>) {
    super();
    if (options) Object.assign(this, options);
    this.source = source;
  }

  /**
  * Create a `Include` from an object
  */
  static from(other: Include): Include {
    return new Include(other.source!, other);
  }
}
