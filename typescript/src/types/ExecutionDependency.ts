// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from './Entity';
import { ExecutionDependencyNode } from './ExecutionDependencyNode';
import { ExecutionDependencyRelation } from './ExecutionDependencyRelation';
import { Integer } from './Integer';

// An upstream execution dependency of a node
export class ExecutionDependency extends Entity {
  type = "ExecutionDependency";

  // The relation to the dependency
  dependencyRelation: ExecutionDependencyRelation;

  // The node that is the dependency
  dependencyNode: ExecutionDependencyNode;

  // The location that the dependency is defined within code
  codeLocation?: Integer[];

  constructor(dependencyRelation: ExecutionDependencyRelation, dependencyNode: ExecutionDependencyNode, options?: ExecutionDependency) {
    super()
    if (options) Object.assign(this, options)
    this.dependencyRelation = dependencyRelation;
    this.dependencyNode = dependencyNode;
  }

  static from(other: ExecutionDependency): ExecutionDependency {
    return new ExecutionDependency(other.dependencyRelation!, other.dependencyNode!, other)
  }
}
