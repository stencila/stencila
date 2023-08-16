// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Primitive } from './Primitive';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// A property-value pair.
export class PropertyValue {
  // The type of this item
  type = "PropertyValue";

  // The identifier for this item
  id?: String;

  // Alternate names (aliases) for the item.
  alternateNames?: String[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name?: String;

  // The URL of the item.
  url?: String;

  // A commonly used identifier for the characteristic represented by the property.
  propertyID?: String;

  // The value of the property.
  value: Primitive;

  constructor(value: Primitive, options?: PropertyValue) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
