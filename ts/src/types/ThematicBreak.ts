// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
 */
export class ThematicBreak extends Entity {
  type = "ThematicBreak";

  constructor(options?: Partial<ThematicBreak>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ThematicBreak`
*/
export function thematicBreak(options?: Partial<ThematicBreak>): ThematicBreak {
  return new ThematicBreak(options);
}
