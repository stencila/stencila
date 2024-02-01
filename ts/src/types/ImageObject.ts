// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { MediaObject } from "./MediaObject.js";

/**
 * An image file.
 */
export class ImageObject extends MediaObject {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ImageObject";

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
    this.type = "ImageObject";
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
