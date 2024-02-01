// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Node } from "./Node.js";

/**
 * A variable representing a name / value pair.
 */
export class Variable extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Variable";

  /**
   * The name of the variable.
   */
  name: string;

  /**
   * The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`
   */
  kind?: string;

  /**
   * The value of the variable.
   */
  value?: Node;

  constructor(name: string, options?: Partial<Variable>) {
    super();
    this.type = "Variable";
    if (options) Object.assign(this, options);
    this.name = name;
  }
}

/**
* Create a new `Variable`
*/
export function variable(name: string, options?: Partial<Variable>): Variable {
  return new Variable(name, options);
}
