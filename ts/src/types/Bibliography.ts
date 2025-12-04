// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Executable } from "./Executable.js";
import { Reference } from "./Reference.js";

/**
 * A bibliography loaded from an external source file.
 */
export class Bibliography extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Bibliography";

  /**
   * The external source of the bibliography, a file path or URL.
   */
  source: string;

  /**
   * Media type of the source content.
   */
  mediaType?: string;

  /**
   * The references loaded from the source.
   */
  references?: Reference[];

  constructor(source: string, options?: Partial<Bibliography>) {
    super();
    this.type = "Bibliography";
    if (options) Object.assign(this, options);
    this.source = source;
  }
}

/**
* Create a new `Bibliography`
*/
export function bibliography(source: string, options?: Partial<Bibliography>): Bibliography {
  return new Bibliography(source, options);
}
