// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ExecutionDependantNode } from './ExecutionDependantNode';
import { ExecutionDependantRelation } from './ExecutionDependantRelation';
import { Integer } from './Integer';

// A downstream execution dependant of a node
export class ExecutionDependant {
  // The relation to the dependant
  dependantRelation: ExecutionDependantRelation;

  // The node that is the dependant
  dependantNode: ExecutionDependantNode;

  // The location that the dependant is defined within code
  codeLocation?: Integer[];

  constructor(dependantRelation: ExecutionDependantRelation, dependantNode: ExecutionDependantNode, options?: ExecutionDependant) {
    if (options) Object.assign(this, options)
    this.dependantRelation = dependantRelation;
    this.dependantNode = dependantNode;
  }
}
