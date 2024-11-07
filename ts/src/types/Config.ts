// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * Stencila document configuration options.
 */
export class Config extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Config";

  /**
   * The styling theme to use for the document
   */
  theme?: string;

  constructor(options?: Partial<Config>) {
    super();
    this.type = "Config";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Config`
*/
export function config(options?: Partial<Config>): Config {
  return new Config(options);
}
