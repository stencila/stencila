// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { CreativeWorkType } from "./CreativeWorkType.js";
import { CreativeWorkVariant } from "./CreativeWorkVariant.js";
import { Entity } from "./Entity.js";

/**
 * A supplementary `CreativeWork` that supports this work but is not considered part of its main content.
 */
export class Supplement extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Supplement";

  /**
   * The `CreativeWork` type of the supplement.
   */
  workType?: CreativeWorkType;

  /**
   * A short identifier or title for the supplement (e.g., "S1").
   */
  label?: string;

  /**
   * Whether the supplement label should be automatically generated and updated.
   */
  labelAutomatically?: boolean;

  /**
   * A brief caption or description for the supplement.
   */
  caption?: Block[];

  /**
   * A reference to the supplement.
   */
  target?: string;

  /**
   * Any messages generated while embedding the supplement.
   */
  compilationMessages?: CompilationMessage[];

  /**
   * The `CreativeWork` that constitutes the supplement.
   */
  work?: CreativeWorkVariant;

  constructor(options?: Partial<Supplement>) {
    super();
    this.type = "Supplement";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Supplement`
*/
export function supplement(options?: Partial<Supplement>): Supplement {
  return new Supplement(options);
}
