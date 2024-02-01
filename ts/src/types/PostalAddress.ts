// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ContactPoint } from "./ContactPoint.js";

/**
 * A physical mailing address.
 */
export class PostalAddress extends ContactPoint {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "PostalAddress";

  /**
   * The street address.
   */
  streetAddress?: string;

  /**
   * The post office box number.
   */
  postOfficeBoxNumber?: string;

  /**
   * The locality in which the street address is, and which is in the region.
   */
  addressLocality?: string;

  /**
   * The region in which the locality is, and which is in the country.
   */
  addressRegion?: string;

  /**
   * The postal code.
   */
  postalCode?: string;

  /**
   * The country.
   */
  addressCountry?: string;

  constructor(options?: Partial<PostalAddress>) {
    super();
    this.type = "PostalAddress";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `PostalAddress`
*/
export function postalAddress(options?: Partial<PostalAddress>): PostalAddress {
  return new PostalAddress(options);
}
