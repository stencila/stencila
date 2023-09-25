// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

// A calendar date encoded as a ISO 8601 string.
export class Date extends Entity {
  type = "Date";

  // The date as an ISO 8601 string.
  value: string;

  constructor(value: string, options?: Date) {
    super();
    if (options) Object.assign(this, options);
    this.value = value;
  }

  static from(other: Date): Date {
    return new Date(other.value!, other);
  }
}
