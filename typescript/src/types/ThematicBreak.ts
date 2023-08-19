// Generated file; do not edit. See `../rust/schema-gen` crate.

import { String } from './String';

// A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
export class ThematicBreak {
  // The type of this item
  type = "ThematicBreak";

  // The identifier for this item
  id?: String;

  constructor(options?: ThematicBreak) {
    if (options) Object.assign(this, options)
    
  }
}
