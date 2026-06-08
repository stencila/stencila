// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { CreativeWork } from "./CreativeWork.js";
import { Duration } from "./Duration.js";
import { ExecutionMessage } from "./ExecutionMessage.js";
import { ExecutionMode } from "./ExecutionMode.js";
import { ExecutionRequired } from "./ExecutionRequired.js";
import { ExecutionStatus } from "./ExecutionStatus.js";
import { ExecutionTag } from "./ExecutionTag.js";
import { Integer } from "./Integer.js";
import { IntegerOrString } from "./IntegerOrString.js";
import { List } from "./List.js";
import { Node } from "./Node.js";
import { type Object } from "./Object.js";
import { Timestamp } from "./Timestamp.js";

/**
 * An article, including news and scholarly articles.
 */
export class Article extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Article";

  /**
   * Under which circumstances the node should be executed.
   */
  executionMode?: ExecutionMode;

  /**
   * A digest of the content, semantics and dependencies of the node.
   */
  compilationDigest?: CompilationDigest;

  /**
   * Messages generated while compiling the code.
   */
  compilationMessages?: CompilationMessage[];

  /**
   * The `compilationDigest` of the node when it was last executed.
   */
  executionDigest?: CompilationDigest;

  /**
   * Tags in the code which affect its execution.
   */
  executionTags?: ExecutionTag[];

  /**
   * A count of the number of times that the node has been executed.
   */
  executionCount?: Integer;

  /**
   * Whether, and why, the code requires execution or re-execution.
   */
  executionRequired?: ExecutionRequired;

  /**
   * Status of the most recent, including any current, execution.
   */
  executionStatus?: ExecutionStatus;

  /**
   * The id of the kernel instance that performed the last execution.
   */
  executionInstance?: string;

  /**
   * The timestamp when the last execution ended.
   */
  executionEnded?: Timestamp;

  /**
   * Duration of the last execution.
   */
  executionDuration?: Duration;

  /**
   * Messages emitted while executing the node.
   */
  executionMessages?: ExecutionMessage[];

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
   * Frontmatter containing document metadata.
   */
  frontmatter?: string;

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
   * Additional metadata for the article.
   */
  extra?: Object;

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
