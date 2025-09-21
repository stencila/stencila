// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ConfigModels } from "./ConfigModels.js";
import { ConfigPublish } from "./ConfigPublish.js";

/**
 * Stencila document configuration options.
 */
export class Config {
  /**
   * The styling theme for the document
   */
  theme?: string;

  /**
   * The citation style for the document (e.g. "APA", "Vancouver")
   */
  citationStyle?: string;

  /**
   * The parameters used for selecting and running generative AI models
   */
  models?: ConfigModels;

  /**
   * Publishing configuration options
   */
  publish?: ConfigPublish;

  constructor(options?: Partial<Config>) {
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Config`
*/
export function config(options?: Partial<Config>): Config {
  return new Config(options);
}
