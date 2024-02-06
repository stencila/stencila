// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { MediaObject } from "./MediaObject.js";

/**
 * An audio file.
 */
export class AudioObject extends MediaObject {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "AudioObject";

  /**
   * The caption for this audio recording.
   */
  caption?: Inline[];

  /**
   * The transcript of this audio recording.
   */
  transcript?: string;

  constructor(contentUrl: string, options?: Partial<AudioObject>) {
    super(contentUrl);
    this.type = "AudioObject";
    if (options) Object.assign(this, options);
    this.contentUrl = contentUrl;
  }
}

/**
* Create a new `AudioObject`
*/
export function audioObject(contentUrl: string, options?: Partial<AudioObject>): AudioObject {
  return new AudioObject(contentUrl, options);
}
