// Generated file; do not edit. See `../rust/schema-gen` crate.

// A point in time recurring on multiple days
export class Time {
  type = "Time";

  // The identifier for this item
  id?: string;

  // The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`.
  value: string;

  constructor(value: string, options?: Time) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
