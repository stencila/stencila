// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Strongly emphasized content.
 */
export class Strong extends Mark {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Strong";

  constructor(content: Inline[], options?: Partial<Strong>) {
    super(content);
    this.type = "Strong";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Strong`
*/
export function strong(content: Inline[], options?: Partial<Strong>): Strong {
  return new Strong(content, options);
}
