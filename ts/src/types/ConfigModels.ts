// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ExecutionBounds } from "./ExecutionBounds.js";

/**
 * Model selection and execution options.
 */
export class ConfigModels {
  /**
   * Automatically execute generated content.
   */
  executeContent?: boolean;

  /**
   * The execution boundaries on model generated code.
   */
  executionBounds?: ExecutionBounds;

  /**
   * When executing model generated content, the maximum number of retries.
   */
  maximumRetries?: number;

  constructor(options?: Partial<ConfigModels>) {
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ConfigModels`
*/
export function configModels(options?: Partial<ConfigModels>): ConfigModels {
  return new ConfigModels(options);
}
