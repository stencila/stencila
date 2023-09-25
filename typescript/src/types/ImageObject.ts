// Generated file; do not edit. See `../rust/schema-gen` crate.

import { MediaObject } from "./MediaObject.js";

/**
 * An image file.
 */
export class ImageObject extends MediaObject {
  type = "ImageObject";

  /**
   * The caption for this image.
   */
  caption?: string;

  /**
   * Thumbnail image of this image.
   */
  thumbnail?: ImageObject;

  constructor(contentUrl: string, options?: Partial<ImageObject>) {
    super(contentUrl);
    if (options) Object.assign(this, options);
    this.contentUrl = contentUrl;
  }

  /**
  * Create a `ImageObject` from an object
  */
  static from(other: ImageObject): ImageObject {
    return new ImageObject(other.contentUrl!, other);
  }
}
