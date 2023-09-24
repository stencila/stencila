// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Primitive } from './Primitive';
import { Thing } from './Thing';

// A property-value pair.
export class PropertyValue extends Thing {
  type = "PropertyValue";

  // A commonly used identifier for the characteristic represented by the property.
  propertyID?: string;

  // The value of the property.
  value: Primitive;

  constructor(value: Primitive, options?: PropertyValue) {
    super()
    if (options) Object.assign(this, options)
    this.value = value;
  }

  static from(other: PropertyValue): PropertyValue {
    return new PropertyValue(other.value!, other)
  }
}
