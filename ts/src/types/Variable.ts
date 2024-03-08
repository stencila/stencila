// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Hint } from "./Hint.js";
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
   * The programming language that the variable is defined in e.g. Python, JSON.
   */
  programmingLanguage?: string;

  /**
   * The native type of the variable e.g. `float`, `datetime.datetime`, `pandas.DataFrame`
   */
  nativeType?: string;

  /**
   * The Stencila node type of the variable e.g. `Number`, `DateTime`, `Datatable`.
   */
  nodeType?: string;

  /**
   * The value of the variable.
   */
  value?: Node;

  /**
   * A hint to the value and/or structure of the variable.
   */
  hint?: Hint;

  /**
   * A textual hint to the value and/or structure of the variable.
   */
  nativeHint?: string;

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
