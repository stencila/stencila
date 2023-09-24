// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';

// A tag on code that affects its execution
export class ExecutionTag extends Entity {
  type = "ExecutionTag";

  // The name of the tag
  name: string;

  // The value of the tag
  value: string;

  // Whether the tag is global to the document
  isGlobal: boolean;

  constructor(name: string, value: string, isGlobal: boolean, options?: ExecutionTag) {
    super()
    if (options) Object.assign(this, options)
    this.name = name;
    this.value = value;
    this.isGlobal = isGlobal;
  }

  static from(other: ExecutionTag): ExecutionTag {
    return new ExecutionTag(other.name!, other.value!, other.isGlobal!, other)
  }
}
