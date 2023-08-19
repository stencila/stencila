// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Parameter } from './Parameter';
import { Validator } from './Validator';

// A function with a name, which might take Parameters and return a value of a certain type.
export class Function {
  type = "Function";

  // The identifier for this item
  id?: string;

  // The name of the function.
  name: string;

  // The parameters of the function.
  parameters: Parameter[];

  // The return type of the function.
  returns?: Validator;

  constructor(name: string, parameters: Parameter[], options?: Function) {
    if (options) Object.assign(this, options)
    this.name = name;
    this.parameters = parameters;
  }
}
