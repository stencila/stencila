// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ActionAgent } from "./ActionAgent.js";
import { ActionStatusType } from "./ActionStatusType.js";
import { ContainerImageOrString } from "./ContainerImageOrString.js";
import { DateTime } from "./DateTime.js";
import { Node } from "./Node.js";
import { PropertyValue } from "./PropertyValue.js";
import { Thing } from "./Thing.js";
import { ThingVariantOrString } from "./ThingVariantOrString.js";

/**
 * An action performed by an agent.
 */
export class Action extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Action";

  /**
   * The current status of the action.
   */
  actionStatus?: ActionStatusType;

  /**
   * The direct performer or driver of the action.
   */
  agent?: ActionAgent;

  /**
   * Other agents that participated in the action.
   */
  participants?: ActionAgent[];

  /**
   * The service provider, service operator, or performer responsible for the action.
   */
  provider?: ActionAgent;

  /**
   * The objects or input values upon which the action is carried out.
   */
  objects?: Node[];

  /**
   * The objects or values produced by the action.
   */
  results?: Node[];

  /**
   * The object, software, or other instrument that helped perform the action.
   */
  instrument?: ThingVariantOrString;

  /**
   * Environment variables or settings that affected the action.
   */
  environment?: PropertyValue[];

  /**
   * Container images used by the action.
   */
  containerImages?: ContainerImageOrString[];

  /**
   * When the action started.
   */
  startTime?: DateTime;

  /**
   * When the action ended.
   */
  endTime?: DateTime;

  /**
   * An error produced by the action.
   */
  error?: ThingVariantOrString;

  constructor(options?: Partial<Action>) {
    super();
    this.type = "Action";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Action`
*/
export function action(options?: Partial<Action>): Action {
  return new Action(options);
}
