// Generated file; do not edit. See `../rust/schema-gen` crate.

// A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
export class DateTime {
  type = "DateTime";

  // The identifier for this item
  id?: string;

  // The date as an ISO 8601 string.
  value: string;

  constructor(value: string, options?: DateTime) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
