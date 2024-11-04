// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";

/**
 * A step in a walkthrough.
 */
export class WalkthroughStep extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "WalkthroughStep";

  /**
   * Whether this step is active (i.e. is encoded in source format and can be edited)
   */
  isCollapsed?: boolean;

  /**
   * The content of the step.
   */
  content: Block[];

  constructor(content: Block[], options?: Partial<WalkthroughStep>) {
    super();
    this.type = "WalkthroughStep";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `WalkthroughStep`
*/
export function walkthroughStep(content: Block[], options?: Partial<WalkthroughStep>): WalkthroughStep {
  return new WalkthroughStep(content, options);
}
