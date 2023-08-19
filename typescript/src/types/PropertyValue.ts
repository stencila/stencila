// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Primitive } from './Primitive';
import { PropertyValueOrString } from './PropertyValueOrString';

// A property-value pair.
export class PropertyValue {
  type = "PropertyValue";

  // The identifier for this item
  id?: string;

  // Alternate names (aliases) for the item.
  alternateNames?: string[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name?: string;

  // The URL of the item.
  url?: string;

  // A commonly used identifier for the characteristic represented by the property.
  propertyID?: string;

  // The value of the property.
  value: Primitive;

  constructor(value: Primitive, options?: PropertyValue) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
