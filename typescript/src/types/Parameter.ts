// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Executable } from './Executable';
import { Node } from './Node';
import { Validator } from './Validator';

// A parameter of a document.
export class Parameter extends Executable {
  type = "Parameter";

  // The name of the parameter.
  name: string;

  // A short label for the parameter.
  label?: string;

  // The current value of the parameter.
  value?: Node;

  // The default value of the parameter.
  default?: Node;

  // The validator that the value is validated against.
  validator?: Validator;

  // Whether the parameter should be hidden.
  hidden?: boolean;

  // The dotted path to the object (e.g. a database table column) that the parameter should be derived from
  derivedFrom?: string;

  constructor(name: string, options?: Parameter) {
    super()
    if (options) Object.assign(this, options)
    this.name = name;
  }
}
