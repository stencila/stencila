// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ExecutionDependencyNode } from './ExecutionDependencyNode';
import { ExecutionDependencyRelation } from './ExecutionDependencyRelation';
import { Integer } from './Integer';

// An upstream execution dependency of a node
export class ExecutionDependency {
  // The relation to the dependency
  dependencyRelation: ExecutionDependencyRelation;

  // The node that is the dependency
  dependencyNode: ExecutionDependencyNode;

  // The location that the dependency is defined within code
  codeLocation?: Integer[];

  constructor(dependencyRelation: ExecutionDependencyRelation, dependencyNode: ExecutionDependencyNode, options?: ExecutionDependency) {
    if (options) Object.assign(this, options)
    this.dependencyRelation = dependencyRelation;
    this.dependencyNode = dependencyNode;
  }
}
