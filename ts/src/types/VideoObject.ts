// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ImageObject } from "./ImageObject.js";
import { Inline } from "./Inline.js";
import { MediaObject } from "./MediaObject.js";

/**
 * A video file.
 */
export class VideoObject extends MediaObject {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "VideoObject";

  /**
   * The caption for this video recording.
   */
  caption?: Inline[];

  /**
   * Thumbnail image of this video recording.
   */
  thumbnail?: ImageObject;

  /**
   * The transcript of this video recording.
   */
  transcript?: string;

  constructor(contentUrl: string, options?: Partial<VideoObject>) {
    super(contentUrl);
    this.type = "VideoObject";
    if (options) Object.assign(this, options);
    this.contentUrl = contentUrl;
  }
}

/**
* Create a new `VideoObject`
*/
export function videoObject(contentUrl: string, options?: Partial<VideoObject>): VideoObject {
  return new VideoObject(contentUrl, options);
}
