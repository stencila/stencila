// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Integer } from './Integer';

// A schema specifying constraints on a string node.
export class StringValidator {
  type = "StringValidator";

  // The identifier for this item
  id?: string;

  // The minimum length for a string node.
  minLength?: Integer;

  // The maximum length for a string node.
  maxLength?: Integer;

  // A regular expression that a string node must match.
  pattern?: string;

  constructor(options?: StringValidator) {
    if (options) Object.assign(this, options)
    
  }
}
