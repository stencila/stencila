// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Node } from './Node';
import { String } from './String';

// A variable representing a name / value pair.
export class Variable {
  // The type of this item
  type = "Variable";

  // The identifier for this item
  id?: String;

  // The namespace, usually a document path, within which the variable resides
  namespace: String;

  // The name of the variable.
  name: String;

  // The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`
  kind?: String;

  // The value of the variable.
  value?: Node;

  constructor(namespace: String, name: String, options?: Variable) {
    if (options) Object.assign(this, options)
    this.namespace = namespace;
    this.name = name;
  }
}
