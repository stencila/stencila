// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";
import { ExecutionDependencyNode } from "./ExecutionDependencyNode.js";
import { ExecutionDependencyRelation } from "./ExecutionDependencyRelation.js";

/**
 * An upstream execution dependency of a node.
 */
export class ExecutionDependency extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ExecutionDependency";

  /**
   * The relation to the dependency.
   */
  dependencyRelation: ExecutionDependencyRelation;

  /**
   * The node that is the dependency.
   */
  dependencyNode: ExecutionDependencyNode;

  /**
   * The location that the dependency is defined.
   */
  codeLocation?: CodeLocation;

  constructor(dependencyRelation: ExecutionDependencyRelation, dependencyNode: ExecutionDependencyNode, options?: Partial<ExecutionDependency>) {
    super();
    this.type = "ExecutionDependency";
    if (options) Object.assign(this, options);
    this.dependencyRelation = dependencyRelation;
    this.dependencyNode = dependencyNode;
  }
}

/**
* Create a new `ExecutionDependency`
*/
export function executionDependency(dependencyRelation: ExecutionDependencyRelation, dependencyNode: ExecutionDependencyNode, options?: Partial<ExecutionDependency>): ExecutionDependency {
  return new ExecutionDependency(dependencyRelation, dependencyNode, options);
}
