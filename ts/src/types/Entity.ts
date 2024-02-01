// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

/**
 * Abstract base type for compound (ie. non-atomic) nodes.
 */
export class Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Entity";

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
