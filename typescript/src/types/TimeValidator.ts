// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { Time } from './Time';

// A validator specifying the constraints on a time.
export class TimeValidator extends Entity {
  type = "TimeValidator";

  // The inclusive lower limit for a time.
  minimum?: Time;

  // The inclusive upper limit for a time.
  maximum?: Time;

  constructor(options?: TimeValidator) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
