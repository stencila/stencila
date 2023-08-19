// Generated file; do not edit. See `../rust/schema-gen` crate.

// A calendar date encoded as a ISO 8601 string.
export class Date {
  type = "Date";

  // The identifier for this item
  id?: string;

  // The date as an ISO 8601 string.
  value: string;

  constructor(value: string, options?: Date) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
