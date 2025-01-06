// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * Model selection and inference parameters for generative AI models.
 */
export class ModelParameters extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ModelParameters";

  /**
   * The ids of the models to select.
   */
  modelIds?: string[];

  /**
   * The number of replicate inferences to run per model id.
   */
  replicates?: UnsignedInteger;

  /**
   * The relative weighting given to model quality (0-100).
   */
  qualityWeight?: UnsignedInteger;

  /**
   * The relative weighting given to model cost (0-100).
   */
  costWeight?: UnsignedInteger;

  /**
   * The relative weighting given to model speed (0-100).
   */
  speedWeight?: UnsignedInteger;

  /**
   * The minimum score for models to be selected (0-100).
   */
  minimumScore?: UnsignedInteger;

  /**
   * The temperature option for model inference (0-100).
   */
  temperature?: UnsignedInteger;

  /**
   * The random seed used for the model (if possible)
   */
  randomSeed?: Integer;

  constructor(options?: Partial<ModelParameters>) {
    super();
    this.type = "ModelParameters";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ModelParameters`
*/
export function modelParameters(options?: Partial<ModelParameters>): ModelParameters {
  return new ModelParameters(options);
}
