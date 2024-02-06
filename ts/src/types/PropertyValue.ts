// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Primitive } from "./Primitive.js";
import { Thing } from "./Thing.js";

/**
 * A property-value pair.
 */
export class PropertyValue extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "PropertyValue";

  /**
   * A commonly used identifier for the characteristic represented by the property.
   */
  propertyID?: string;

  /**
   * The value of the property.
   */
  value: Primitive;

  constructor(value: Primitive, options?: Partial<PropertyValue>) {
    super();
    this.type = "PropertyValue";
    if (options) Object.assign(this, options);
    this.value = value;
  }
}

/**
* Create a new `PropertyValue`
*/
export function propertyValue(value: Primitive, options?: Partial<PropertyValue>): PropertyValue {
  return new PropertyValue(value, options);
}
