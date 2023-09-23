// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from './CreativeWork';
import { Thing } from './Thing';

// A review of an item, e.g of an Article, or SoftwareSourceCode.
export class Review extends CreativeWork {
  type = "Review";

  // The item that is being reviewed.
  itemReviewed?: Thing;

  // The part or facet of the item that is being reviewed.
  reviewAspect?: string;

  constructor(options?: Review) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
