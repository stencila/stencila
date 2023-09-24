// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CitationIntent } from './CitationIntent';
import { CitationMode } from './CitationMode';
import { Entity } from './Entity';
import { Inline } from './Inline';
import { IntegerOrString } from './IntegerOrString';

// A reference to a CreativeWork that is cited in another CreativeWork.
export class Cite extends Entity {
  type = "Cite";

  // The target of the citation (URL or reference ID).
  target: string;

  // Determines how the citation is shown within the surrounding text.
  citationMode: CitationMode;

  // The type/s of the citation, both factually and rhetorically.
  citationIntent?: CitationIntent[];

  // Optional structured content/text of this citation.
  content?: Inline[];

  // The page on which the work starts; for example "135" or "xiii".
  pageStart?: IntegerOrString;

  // The page on which the work ends; for example "138" or "xvi".
  pageEnd?: IntegerOrString;

  // Any description of pages that is not separated into pageStart and pageEnd;
  // for example, "1-6, 9, 55".
  pagination?: string;

  // Text to show before the citation.
  citationPrefix?: string;

  // Text to show after the citation.
  citationSuffix?: string;

  constructor(target: string, citationMode: CitationMode, options?: Cite) {
    super()
    if (options) Object.assign(this, options)
    this.target = target;
    this.citationMode = citationMode;
  }

  static from(other: Cite): Cite {
    return new Cite(other.target!, other.citationMode!, other)
  }
}
