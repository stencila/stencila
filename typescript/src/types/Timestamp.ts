// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Integer } from './Integer';
import { String } from './String';
import { TimeUnit } from './TimeUnit';

// A value that represents a point in time
export class Timestamp {
  // The type of this item
  type = "Timestamp";

  // The identifier for this item
  id?: String;

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
