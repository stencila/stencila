// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cite } from './Cite';
import { Entity } from './Entity';

// A group of Cite nodes.
export class CiteGroup extends Entity {
  type = "CiteGroup";

  // One or more `Cite`s to be referenced in the same surrounding text.
  items: Cite[];

  constructor(items: Cite[], options?: CiteGroup) {
    super()
    if (options) Object.assign(this, options)
    this.items = items;
  }
}
