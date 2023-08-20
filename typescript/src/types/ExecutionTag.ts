// Generated file; do not edit. See `../rust/schema-gen` crate.

// A tag on code that affects its execution
export class ExecutionTag {
  type = "ExecutionTag";

  // The identifier for this item
  id?: string;

  // The name of the tag
  name: string;

  // The value of the tag
  value: string;

  // Whether the tag is global to the document
  isGlobal: boolean;

  constructor(name: string, value: string, isGlobal: boolean, options?: ExecutionTag) {
    if (options) Object.assign(this, options)
    this.name = name;
    this.value = value;
    this.isGlobal = isGlobal;
  }
}
