// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// A brand used by an organization or person for labeling a product, product group, or similar.
export class Brand {
  // The type of this item
  type = "Brand";

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
  name: String;

  // The URL of the item.
  url?: String;

  // A logo associated with the brand.
  logo?: ImageObjectOrString;

  // Reviews of the brand.
  reviews?: String[];

  constructor(name: String, options?: Brand) {
    if (options) Object.assign(this, options)
    this.name = name;
  }
}
