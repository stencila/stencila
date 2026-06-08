// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

/**
 * Abstract base for document nodes with labels.
 */
export class Labelled {
  /**
   * Whether the identifier should be automatically updated.
   */
  idAutomatically?: boolean;

  /**
   * A short label for the node.
   */
  label?: string;

  /**
   * Whether the label should be automatically updated.
   */
  labelAutomatically?: boolean;

  constructor(options?: Partial<Labelled>) {
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Labelled`
*/
export function labelled(options?: Partial<Labelled>): Labelled {
  return new Labelled(options);
}
