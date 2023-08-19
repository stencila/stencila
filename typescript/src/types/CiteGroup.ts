// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cite } from './Cite';
import { String } from './String';

// A group of Cite nodes.
export class CiteGroup {
  // The type of this item
  type = "CiteGroup";

  // The identifier for this item
  id?: String;

  // One or more `Cite`s to be referenced in the same surrounding text.
  items: Cite[];

  constructor(items: Cite[], options?: CiteGroup) {
    if (options) Object.assign(this, options)
    this.items = items;
  }
}
