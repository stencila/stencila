// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Parameter } from "./Parameter.js";
import { Validator } from "./Validator.js";

/**
 * A function with a name, which might take Parameters and return a value of a certain type.
 */
export class Function extends Entity {
  type = "Function";

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
