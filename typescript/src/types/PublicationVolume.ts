// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from './CreativeWork';
import { IntegerOrString } from './IntegerOrString';

// A part of a successively published publication such as a periodical or multi-volume work.
export class PublicationVolume extends CreativeWork {
  type = "PublicationVolume";

  // The page on which the volume starts; for example "135" or "xiii".
  pageStart?: IntegerOrString;

  // The page on which the volume ends; for example "138" or "xvi".
  pageEnd?: IntegerOrString;

  // Any description of pages that is not separated into pageStart and pageEnd;
  // for example, "1-6, 9, 55".
  pagination?: string;

  // Identifies the volume of publication or multi-part work; for example, "iii" or "2".
  volumeNumber?: IntegerOrString;

  constructor(options?: PublicationVolume) {
    super()
    if (options) Object.assign(this, options)
    
  }

  static from(other: PublicationVolume): PublicationVolume {
    return new PublicationVolume(other)
  }
}
