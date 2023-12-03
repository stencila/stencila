// Generated file; do not edit. See `../rust/schema-gen` crate.

/**
 * Abstract base type for compound (ie. non-atomic) nodes.
 */
export class Entity {
  type = "Entity";

  /**
   * The identifier for this item.
   */
  id?: string;

  constructor(options?: Partial<Entity>) {
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Entity`
*/
export function entity(options?: Partial<Entity>): Entity {
  return new Entity(options);
}
