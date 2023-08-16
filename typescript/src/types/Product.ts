// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Block } from './Block';
import { Brand } from './Brand';
import { ImageObjectOrString } from './ImageObjectOrString';
import { PropertyValueOrString } from './PropertyValueOrString';
import { String } from './String';

// Any offered product or service. For example, a pair of shoes;
  // a haircut; or an episode of a TV show streamed online.
export class Product {
  // The type of this item
  type = "Product";

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

  // Brands that the product is labelled with.
  brands?: Brand[];

  // The logo of the product.
  logo?: ImageObjectOrString;

  // Product identification code.
  productID?: String;

  constructor(options?: Product) {
    if (options) Object.assign(this, options)
    
  }
}
