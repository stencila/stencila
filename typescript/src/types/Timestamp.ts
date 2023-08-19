// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Integer } from './Integer';
import { TimeUnit } from './TimeUnit';

// A value that represents a point in time
export class Timestamp {
  type = "Timestamp";

  // The identifier for this item
  id?: string;

  // The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z).
  value: Integer;

  // The time unit that the `value` represents.
  timeUnit: TimeUnit;

  constructor(value: Integer, timeUnit: TimeUnit, options?: Timestamp) {
    if (options) Object.assign(this, options)
    this.value = value;
    this.timeUnit = timeUnit;
  }
}
