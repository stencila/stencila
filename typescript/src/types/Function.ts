// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Parameter } from './Parameter';
import { String } from './String';
import { Validator } from './Validator';

// A function with a name, which might take Parameters and return a value of a certain type.
export class Function {
  // The type of this item
  type = "Function";

  // The identifier for this item
  id?: String;

  // The name of the function.
  name: String;

  // The parameters of the function.
  parameters: Parameter[];

  // The return type of the function.
  returns?: Validator;

  constructor(name: String, parameters: Parameter[], options?: Function) {
    if (options) Object.assign(this, options)
    this.name = name;
    this.parameters = parameters;
  }
}
