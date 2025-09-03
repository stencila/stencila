// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";
import { ExecutionDependantRelation } from "./ExecutionDependantRelation.js";

/**
 * A downstream execution dependant of a node.
 */
export class ExecutionDependant extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ExecutionDependant";

  /**
   * The relation to the dependant.
   */
  dependantRelation: ExecutionDependantRelation;

  /**
   * The type of node that is the dependant.
   */
  dependantType: string;

  /**
   * The id of node that is the dependant.
   */
  dependantId: string;

  /**
   * The location that the dependant is defined.
   */
  codeLocation?: CodeLocation;

  constructor(dependantRelation: ExecutionDependantRelation, dependantType: string, dependantId: string, options?: Partial<ExecutionDependant>) {
    super();
    this.type = "ExecutionDependant";
    if (options) Object.assign(this, options);
    this.dependantRelation = dependantRelation;
    this.dependantType = dependantType;
    this.dependantId = dependantId;
  }
}

/**
* Create a new `ExecutionDependant`
*/
export function executionDependant(dependantRelation: ExecutionDependantRelation, dependantType: string, dependantId: string, options?: Partial<ExecutionDependant>): ExecutionDependant {
  return new ExecutionDependant(dependantRelation, dependantType, dependantId, options);
}
