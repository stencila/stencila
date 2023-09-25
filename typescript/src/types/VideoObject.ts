// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ImageObject } from "./ImageObject.js";
import { MediaObject } from "./MediaObject.js";

/**
 * A video file.
 */
export class VideoObject extends MediaObject {
  type = "VideoObject";

  /**
   * The caption for this video recording.
   */
  caption?: string;

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
    if (options) Object.assign(this, options);
    this.contentUrl = contentUrl;
  }

  /**
  * Create a `VideoObject` from an object
  */
  static from(other: VideoObject): VideoObject {
    return new VideoObject(other.contentUrl!, other);
  }
}

/**
* Create a new `VideoObject`
*/
export function videoObject(contentUrl: string, options?: Partial<VideoObject>): VideoObject {
  return new VideoObject(contentUrl, options);
}
