// Generated file; do not edit. See `../rust/schema-gen` crate.

import { MediaObject } from "./MediaObject.js";

/**
 * An audio file
 */
export class AudioObject extends MediaObject {
  type = "AudioObject";

  /**
   * The caption for this audio recording.
   */
  caption?: string;

  /**
   * The transcript of this audio recording.
   */
  transcript?: string;

  constructor(contentUrl: string, options?: Partial<AudioObject>) {
    super(contentUrl);
    if (options) Object.assign(this, options);
    this.contentUrl = contentUrl;
  }

  /**
  * Create a `AudioObject` from an object
  */
  static from(other: AudioObject): AudioObject {
    return new AudioObject(other.contentUrl!, other);
  }
}

/**
* Create a new `AudioObject`
*/
export function audioObject(contentUrl: string, options?: Partial<AudioObject>): AudioObject {
  return new AudioObject(contentUrl, options);
}
