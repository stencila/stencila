// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { ExecutionDependantNode } from './ExecutionDependantNode';
import { ExecutionDependantRelation } from './ExecutionDependantRelation';
import { Integer } from './Integer';

// A downstream execution dependant of a node
export class ExecutionDependant extends Entity {
  type = "ExecutionDependant";

  // The relation to the dependant
  dependantRelation: ExecutionDependantRelation;

  // The node that is the dependant
  dependantNode: ExecutionDependantNode;

  // The location that the dependant is defined within code
  codeLocation?: Integer[];

  constructor(dependantRelation: ExecutionDependantRelation, dependantNode: ExecutionDependantNode, options?: ExecutionDependant) {
    super()
    if (options) Object.assign(this, options)
    this.dependantRelation = dependantRelation;
    this.dependantNode = dependantNode;
  }
}
