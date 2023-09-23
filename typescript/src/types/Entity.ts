// Generated file; do not edit. See `../rust/schema-gen` crate.

// Abstract base type for compound (ie. non-atomic) nodes.
export class Entity {
  type = "Entity";

  // The identifier for this item
  id?: string;

  constructor(options?: Entity) {
    if (options) Object.assign(this, options)
    
  }
}
