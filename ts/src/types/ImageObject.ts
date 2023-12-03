// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { MediaObject } from "./MediaObject.js";

/**
 * An image file.
 */
export class ImageObject extends MediaObject {
  type = "ImageObject";

  /**
   * The caption for this image.
   */
  caption?: Inline[];

  /**
   * Thumbnail image of this image.
   */
  thumbnail?: ImageObject;

  constructor(contentUrl: string, options?: Partial<ImageObject>) {
    super(contentUrl);
    if (options) Object.assign(this, options);
    this.contentUrl = contentUrl;
  }
}

/**
* Create a new `ImageObject`
*/
export function imageObject(contentUrl: string, options?: Partial<ImageObject>): ImageObject {
  return new ImageObject(contentUrl, options);
}
