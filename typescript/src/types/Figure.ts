// Generated file; do not edit. See `../rust/schema-gen` crate.

import { BlocksOrString } from "./BlocksOrString.js";
import { CreativeWork } from "./CreativeWork.js";

/**
 * Encapsulates one or more images, videos, tables, etc, and provides captions and labels for them.
 */
export class Figure extends CreativeWork {
  type = "Figure";

  /**
   * A short label for the figure.
   */
  label?: string;

  /**
   * A caption for the figure.
   */
  caption?: BlocksOrString;

  constructor(options?: Partial<Figure>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Figure`
*/
export function figure(options?: Partial<Figure>): Figure {
  return new Figure(options);
}
