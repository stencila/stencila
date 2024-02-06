// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { StringPatchOrPrimitive } from "./StringPatchOrPrimitive.js";

/**
 * An operation that is part of a suggestion to modify the property of a node.
 */
export class ModifyOperation extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ModifyOperation";

  /**
   * The target property of each node to be modified.
   */
  target: string;

  /**
   * The new value, or string patch, to apply to the target property.
   */
  value: StringPatchOrPrimitive;

  constructor(target: string, value: StringPatchOrPrimitive, options?: Partial<ModifyOperation>) {
    super();
    this.type = "ModifyOperation";
    if (options) Object.assign(this, options);
    this.target = target;
    this.value = value;
  }
}

/**
* Create a new `ModifyOperation`
*/
export function modifyOperation(target: string, value: StringPatchOrPrimitive, options?: Partial<ModifyOperation>): ModifyOperation {
  return new ModifyOperation(target, value, options);
}
