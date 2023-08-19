// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cite } from './Cite';

// A group of Cite nodes.
export class CiteGroup {
  type = "CiteGroup";

  // The identifier for this item
  id?: string;

  // One or more `Cite`s to be referenced in the same surrounding text.
  items: Cite[];

  constructor(items: Cite[], options?: CiteGroup) {
    if (options) Object.assign(this, options)
    this.items = items;
  }
}
