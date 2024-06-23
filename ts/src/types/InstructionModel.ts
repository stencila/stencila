// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * The name and execution options for the generative model used for an instruction.
 */
export class InstructionModel extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InstructionModel";

  /**
   * The name of the model.
   */
  name?: string;

  /**
   * The relative weighting given to model quality (0-100).
   */
  qualityWeight?: UnsignedInteger;

  /**
   * The relative weighting given to model speed (0-100).
   */
  speedWeight?: UnsignedInteger;

  /**
   * The relative weighting given to model cost (0-100).
   */
  costWeight?: UnsignedInteger;

  /**
   * The temperature option for model inference (0-100).
   */
  temperature?: UnsignedInteger;

  /**
   * The random seed used for the model (if possible)
   */
  randomSeed?: Integer;

  constructor(options?: Partial<InstructionModel>) {
    super();
    this.type = "InstructionModel";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `InstructionModel`
*/
export function instructionModel(options?: Partial<InstructionModel>): InstructionModel {
  return new InstructionModel(options);
}
