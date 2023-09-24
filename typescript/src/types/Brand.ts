// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ImageObjectOrString } from './ImageObjectOrString';
import { Thing } from './Thing';

// A brand used by an organization or person for labeling a product, product group, or similar.
export class Brand extends Thing {
  type = "Brand";

  // A logo associated with the brand.
  logo?: ImageObjectOrString;

  // Reviews of the brand.
  reviews?: string[];

  constructor(name: string, options?: Brand) {
    super()
    if (options) Object.assign(this, options)
    this.name = name;
  }

  static from(other: Brand): Brand {
    return new Brand(other.name!, other)
  }
}
