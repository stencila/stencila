// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

/**
 * Zenodo publishing options.
 */
export class ConfigPublishZenodo {
  /**
   * Whether the deposit is embargoed.
   */
  embargoed?: boolean;

  constructor(options?: Partial<ConfigPublishZenodo>) {
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ConfigPublishZenodo`
*/
export function configPublishZenodo(options?: Partial<ConfigPublishZenodo>): ConfigPublishZenodo {
  return new ConfigPublishZenodo(options);
}
