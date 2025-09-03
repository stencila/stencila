// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";
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
   * The type of node that is the dependency.
   */
  dependencyType: string;

  /**
   * The id of node that is the dependency.
   */
  dependencyId: string;

  /**
   * The location that the dependency is defined.
   */
  codeLocation?: CodeLocation;

  constructor(dependencyRelation: ExecutionDependencyRelation, dependencyType: string, dependencyId: string, options?: Partial<ExecutionDependency>) {
    super();
    this.type = "ExecutionDependency";
    if (options) Object.assign(this, options);
    this.dependencyRelation = dependencyRelation;
    this.dependencyType = dependencyType;
    this.dependencyId = dependencyId;
  }
}

/**
* Create a new `ExecutionDependency`
*/
export function executionDependency(dependencyRelation: ExecutionDependencyRelation, dependencyType: string, dependencyId: string, options?: Partial<ExecutionDependency>): ExecutionDependency {
  return new ExecutionDependency(dependencyRelation, dependencyType, dependencyId, options);
}
