// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Primitive } from "./Primitive.js";
import { Thing } from "./Thing.js";

/**
 * A property-value pair.
 */
export class PropertyValue extends Thing {
  type = "PropertyValue";

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
