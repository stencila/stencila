// Generated file; do not edit. See `../rust/schema-gen` crate.

import { String } from './String';

// A calendar date encoded as a ISO 8601 string.
export class Date {
  // The type of this item
  type = "Date";

  // The identifier for this item
  id?: String;

  // The date as an ISO 8601 string.
  value: String;

  constructor(value: String, options?: Date) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
