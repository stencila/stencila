// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Brand } from "./Brand.js";
import { ImageObject } from "./ImageObject.js";
import { Thing } from "./Thing.js";

/**
 * Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
 */
export class Product extends Thing {
  type = "Product";

  /**
   * Brands that the product is labelled with.
   */
  brands?: Brand[];

  /**
   * The logo of the product.
   */
  logo?: ImageObject;

  /**
   * Product identification code.
   */
  productID?: string;

  constructor(options?: Partial<Product>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Product`
*/
export function product(options?: Partial<Product>): Product {
  return new Product(options);
}
