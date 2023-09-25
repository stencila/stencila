// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from "./CreativeWork.js";
import { Thing } from "./Thing.js";

// A review of an item, e.g of an Article, or SoftwareSourceCode.
export class Review extends CreativeWork {
  type = "Review";

  // The item that is being reviewed.
  itemReviewed?: Thing;

  // The part or facet of the item that is being reviewed.
  reviewAspect?: string;

  constructor(options?: Review) {
    super();
    if (options) Object.assign(this, options);
    
  }

  static from(other: Review): Review {
    return new Review(other);
  }
}
