// Generated file. Do not edit; see `rust/schema-gen` crate.

import { String } from './String';
import { Time } from './Time';

// A validator specifying the constraints on a time.
export class TimeValidator {
  // The type of this item
  type = "TimeValidator";

  // The identifier for this item
  id?: String;

  // The inclusive lower limit for a time.
  minimum?: Time;

  // The inclusive upper limit for a time.
  maximum?: Time;

  constructor(options?: TimeValidator) {
    if (options) Object.assign(this, options)
    
  }
}
