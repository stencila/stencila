// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ConfigPublish } from "./ConfigPublish.js";

/**
 * Stencila document configuration options.
 */
export class Config {
  /**
   * The styling theme to use for the document
   */
  theme?: string;

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
