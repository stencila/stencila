// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Executable } from "./Executable.js";
import { Node } from "./Node.js";
import { Validator } from "./Validator.js";

/**
 * A parameter of a document.
 */
export class Parameter extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Parameter";

  /**
   * The name of the parameter.
   */
  name: string;

  /**
   * A short label for the parameter.
   */
  label?: string;

  /**
   * The current value of the parameter.
   */
  value?: Node;

  /**
   * The default value of the parameter.
   */
  default?: Node;

  /**
   * The validator that the value is validated against.
   */
  validator?: Validator;

  /**
   * The dotted path to the object (e.g. a database table column) that the parameter should be derived from
   */
  derivedFrom?: string;

  constructor(name: string, options?: Partial<Parameter>) {
    super();
    this.type = "Parameter";
    if (options) Object.assign(this, options);
    this.name = name;
  }
}

/**
* Create a new `Parameter`
*/
export function parameter(name: string, options?: Partial<Parameter>): Parameter {
  return new Parameter(name, options);
}
