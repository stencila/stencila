// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Config } from "./Config.js";
import { CreativeWork } from "./CreativeWork.js";
import { IntegerOrString } from "./IntegerOrString.js";
import { List } from "./List.js";
import { Node } from "./Node.js";

/**
 * An article, including news and scholarly articles.
 */
export class Article extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Article";

  /**
   * The page on which the article starts; for example "135" or "xiii".
   */
  pageStart?: IntegerOrString;

  /**
   * The page on which the article ends; for example "138" or "xvi".
   */
  pageEnd?: IntegerOrString;

  /**
   * Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
   */
  pagination?: string;

  /**
   * Configuration options for the document.
   */
  config?: Config;

  /**
   * A list of links to headings, including implied section headings, within the document
   */
  headings?: List;

  /**
   * The content of the article.
   */
  content: Block[];

  /**
   * Nodes, usually from within `content` of the article, that have been archived.
   */
  archive?: Node[];

  /**
   * Temporary nodes on document
   */
  temporary?: Node[];

  constructor(content: Block[], options?: Partial<Article>) {
    super();
    this.type = "Article";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Article`
*/
export function article(content: Block[], options?: Partial<Article>): Article {
  return new Article(content, options);
}
