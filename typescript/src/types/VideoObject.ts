// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ImageObject } from './ImageObject';
import { MediaObject } from './MediaObject';

// A video file.
export class VideoObject extends MediaObject {
  type = "VideoObject";

  // The caption for this video recording.
  caption?: string;

  // Thumbnail image of this video recording.
  thumbnail?: ImageObject;

  // The transcript of this video recording.
  transcript?: string;

  constructor(contentUrl: string, options?: VideoObject) {
    super(contentUrl)
    if (options) Object.assign(this, options)
    this.contentUrl = contentUrl;
  }
}
