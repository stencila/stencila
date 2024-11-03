// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { WalkthroughStep } from "./WalkthroughStep.js";

/**
 * An interactive walkthrough made up of several, successively revealed steps.
 */
export class Walkthrough extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Walkthrough";

  /**
   * Whether the walkthrough is expanded so that each step can be edited
   */
  isExpanded?: boolean;

  /**
   * The steps making up the walkthrough.
   */
  steps: WalkthroughStep[];

  constructor(steps: WalkthroughStep[], options?: Partial<Walkthrough>) {
    super();
    this.type = "Walkthrough";
    if (options) Object.assign(this, options);
    this.steps = steps;
  }
}

/**
* Create a new `Walkthrough`
*/
export function walkthrough(steps: WalkthroughStep[], options?: Partial<Walkthrough>): Walkthrough {
  return new Walkthrough(steps, options);
}
