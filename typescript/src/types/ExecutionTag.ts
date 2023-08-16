// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Boolean } from './Boolean';
import { String } from './String';

// A tag on code that affects its execution
export class ExecutionTag {
  // The name of the tag
  name: String;

  // The value of the tag
  value: String;

  // Whether the tag is global to the document
  isGlobal: Boolean;

  constructor(name: String, value: String, isGlobal: Boolean, options?: ExecutionTag) {
    if (options) Object.assign(this, options)
    this.name = name;
    this.value = value;
    this.isGlobal = isGlobal;
  }
}
