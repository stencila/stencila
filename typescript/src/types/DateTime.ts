// Generated file. Do not edit; see `rust/schema-gen` crate.

import { String } from './String';

// A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
export class DateTime {
  // The type of this item
  type = "DateTime";

  // The identifier for this item
  id?: String;

  // The date as an ISO 8601 string.
  value: String;

  constructor(value: String, options?: DateTime) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
