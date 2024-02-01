// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A tag on code that affects its execution.
 */
export class ExecutionTag extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ExecutionTag";

  /**
   * The name of the tag
   */
  name: string;

  /**
   * The value of the tag
   */
  value: string;

  /**
   * Whether the tag is global to the document
   */
  isGlobal: boolean;

  constructor(name: string, value: string, isGlobal: boolean, options?: Partial<ExecutionTag>) {
    super();
    this.type = "ExecutionTag";
    if (options) Object.assign(this, options);
    this.name = name;
    this.value = value;
    this.isGlobal = isGlobal;
  }
}

/**
* Create a new `ExecutionTag`
*/
export function executionTag(name: string, value: string, isGlobal: boolean, options?: Partial<ExecutionTag>): ExecutionTag {
  return new ExecutionTag(name, value, isGlobal, options);
}
