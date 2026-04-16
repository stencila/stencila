// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Brand } from "./Brand.js";
import { ImageObject } from "./ImageObject.js";
import { Thing } from "./Thing.js";

/**
 * A product or service.
 */
export class Product extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Product";

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
    this.type = "Product";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Product`
*/
export function product(options?: Partial<Product>): Product {
  return new Product(options);
}
