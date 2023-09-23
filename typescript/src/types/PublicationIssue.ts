// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from './CreativeWork';
import { IntegerOrString } from './IntegerOrString';

// A part of a successively published publication such as a periodical or publication
  // volume, often numbered.
export class PublicationIssue extends CreativeWork {
  type = "PublicationIssue";

  // Identifies the issue of publication; for example, "iii" or "2".
  issueNumber?: IntegerOrString;

  // The page on which the issue starts; for example "135" or "xiii".
  pageStart?: IntegerOrString;

  // The page on which the issue ends; for example "138" or "xvi".
  pageEnd?: IntegerOrString;

  // Any description of pages that is not separated into pageStart and pageEnd;
  // for example, "1-6, 9, 55".
  pagination?: string;

  constructor(options?: PublicationIssue) {
    super()
    if (options) Object.assign(this, options)
    
  }
}
