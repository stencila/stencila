// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from "./CreativeWork.js";

/**
 * A media object, such as an image, video, or audio object embedded in a web page or a downloadable dataset.
 */
export class MediaObject extends CreativeWork {
  type = "MediaObject";

  /**
   * Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
   */
  bitrate?: number;

  /**
   * File size in megabits (Mbit, Mb).
   */
  contentSize?: number;

  /**
   * URL for the actual bytes of the media object, for example the image file or video file.
   */
  contentUrl: string;

  /**
   * URL that can be used to embed the media on a web page via a specific media player.
   */
  embedUrl?: string;

  /**
   * IANA media type (MIME type).
   */
  mediaType?: string;

  constructor(contentUrl: string, options?: Partial<MediaObject>) {
    super();
    if (options) Object.assign(this, options);
    this.contentUrl = contentUrl;
  }
}

/**
* Create a new `MediaObject`
*/
export function mediaObject(contentUrl: string, options?: Partial<MediaObject>): MediaObject {
  return new MediaObject(contentUrl, options);
}
