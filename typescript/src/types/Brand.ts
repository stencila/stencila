// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';

// A brand used by an organization or person for labeling a product, product group, or similar.
export class Brand {
  type = "Brand";

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
  name: string;

  // The URL of the item.
  url?: string;

  // A logo associated with the brand.
  logo?: ImageObjectOrString;

  // Reviews of the brand.
  reviews?: string[];

  constructor(name: string, options?: Brand) {
    if (options) Object.assign(this, options)
    this.name = name;
  }
}
