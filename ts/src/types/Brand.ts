// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ImageObject } from "./ImageObject.js";
import { Thing } from "./Thing.js";

/**
 * A brand used by an organization or person for labeling a product, product group, or similar.
 */
export class Brand extends Thing {
  type = "Brand";

  /**
   * The name of the item.
   */
  name: string;

  /**
   * A logo associated with the brand.
   */
  logo?: ImageObject;

  /**
   * Reviews of the brand.
   */
  reviews?: string[];

  constructor(name: string, options?: Partial<Brand>) {
    super();
    if (options) Object.assign(this, options);
    this.name = name;
  }
}

/**
* Create a new `Brand`
*/
export function brand(name: string, options?: Partial<Brand>): Brand {
  return new Brand(name, options);
}
