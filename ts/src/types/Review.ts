// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CreativeWork } from "./CreativeWork.js";
import { Thing } from "./Thing.js";

/**
 * A review of an item, e.g of an `Article` or `SoftwareApplication`.
 */
export class Review extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Review";

  /**
   * The item that is being reviewed.
   */
  itemReviewed?: Thing;

  /**
   * The part or facet of the item that is being reviewed.
   */
  reviewAspect?: string;

  constructor(options?: Partial<Review>) {
    super();
    this.type = "Review";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Review`
*/
export function review(options?: Partial<Review>): Review {
  return new Review(options);
}
