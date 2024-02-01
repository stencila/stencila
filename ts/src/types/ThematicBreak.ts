// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
 */
export class ThematicBreak extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ThematicBreak";

  constructor(options?: Partial<ThematicBreak>) {
    super();
    this.type = "ThematicBreak";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ThematicBreak`
*/
export function thematicBreak(options?: Partial<ThematicBreak>): ThematicBreak {
  return new ThematicBreak(options);
}
