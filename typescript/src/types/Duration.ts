// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Integer } from './Integer';
import { TimeUnit } from './TimeUnit';

// A value that represents the difference between two timestamps
export class Duration {
  type = "Duration";

  // The identifier for this item
  id?: string;

  // The time difference in `timeUnit`s.
  value: Integer;

  // The time unit that the `value` represents.
  timeUnit: TimeUnit;

  constructor(value: Integer, timeUnit: TimeUnit, options?: Duration) {
    if (options) Object.assign(this, options)
    this.value = value;
    this.timeUnit = timeUnit;
  }
}
