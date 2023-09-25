// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Node } from "./Node.js";

// A variable representing a name / value pair.
export class Variable extends Entity {
  type = "Variable";

  // The namespace, usually a document path, within which the variable resides
  namespace: string;

  // The name of the variable.
  name: string;

  // The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`
  kind?: string;

  // The value of the variable.
  value?: Node;

  constructor(namespace: string, name: string, options?: Variable) {
    super();
    if (options) Object.assign(this, options);
    this.namespace = namespace;
    this.name = name;
  }

  static from(other: Variable): Variable {
    return new Variable(other.namespace!, other.name!, other);
  }
}
