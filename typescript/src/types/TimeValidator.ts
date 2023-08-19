// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Time } from './Time';

// A validator specifying the constraints on a time.
export class TimeValidator {
  type = "TimeValidator";

  // The identifier for this item
  id?: string;

  // The inclusive lower limit for a time.
  minimum?: Time;

  // The inclusive upper limit for a time.
  maximum?: Time;

  constructor(options?: TimeValidator) {
    if (options) Object.assign(this, options)
    
  }
}
