// Generated file. Do not edit; see `rust/schema-gen` crate.

import { String } from './String';

// A point in time recurring on multiple days
export class Time {
  // The type of this item
  type = "Time";

  // The identifier for this item
  id?: String;

  // The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`.
  value: String;

  constructor(value: String, options?: Time) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
