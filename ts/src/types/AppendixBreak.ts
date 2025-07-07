// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A break in a document indicating the start of a new appendix.
 */
export class AppendixBreak extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "AppendixBreak";

  constructor(options?: Partial<AppendixBreak>) {
    super();
    this.type = "AppendixBreak";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `AppendixBreak`
*/
export function appendixBreak(options?: Partial<AppendixBreak>): AppendixBreak {
  return new AppendixBreak(options);
}
