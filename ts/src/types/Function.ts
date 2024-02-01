// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Parameter } from "./Parameter.js";
import { Validator } from "./Validator.js";

/**
 * A function with a name, which might take Parameters and return a value of a certain type.
 */
export class Function extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Function";

  /**
   * The name of the function.
   */
  name: string;

  /**
   * The parameters of the function.
   */
  parameters: Parameter[];

  /**
   * The return type of the function.
   */
  returns?: Validator;

  constructor(name: string, parameters: Parameter[], options?: Partial<Function>) {
    super();
    this.type = "Function";
    if (options) Object.assign(this, options);
    this.name = name;
    this.parameters = parameters;
  }
}

/**
* Create a new `Function`
*/
export function function_(name: string, parameters: Parameter[], options?: Partial<Function>): Function {
  return new Function(name, parameters, options);
}
