// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

// A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
export class ThematicBreak extends Entity {
  type = "ThematicBreak";

  constructor(options?: ThematicBreak) {
    super();
    if (options) Object.assign(this, options);
    
  }

  static from(other: ThematicBreak): ThematicBreak {
    return new ThematicBreak(other);
  }
}
