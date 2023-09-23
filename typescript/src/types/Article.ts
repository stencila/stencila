// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { CreativeWork } from './CreativeWork';
import { IntegerOrString } from './IntegerOrString';

// An article, including news and scholarly articles.
export class Article extends CreativeWork {
  type = "Article";

  // The page on which the article starts; for example "135" or "xiii".
  pageStart?: IntegerOrString;

  // The page on which the article ends; for example "138" or "xvi".
  pageEnd?: IntegerOrString;

  // Any description of pages that is not separated into pageStart and pageEnd;
  // for example, "1-6, 9, 55".
  pagination?: string;

  constructor(content: Block[], options?: Article) {
    super()
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
