// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Block } from "./Block.js";
import { Comment } from "./Comment.js";
import { CreativeWorkType } from "./CreativeWorkType.js";
import { CreativeWorkTypeOrText } from "./CreativeWorkTypeOrText.js";
import { Date } from "./Date.js";
import { GrantOrMonetaryGrant } from "./GrantOrMonetaryGrant.js";
import { Inline } from "./Inline.js";
import { Person } from "./Person.js";
import { PersonOrOrganization } from "./PersonOrOrganization.js";
import { StringOrNumber } from "./StringOrNumber.js";
import { Text } from "./Text.js";
import { Thing } from "./Thing.js";
import { ThingType } from "./ThingType.js";

/**
 * A creative work, including books, movies, photographs, software programs, etc.
 */
export class CreativeWork extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CreativeWork";

  /**
   * The subject matter of the content.
   */
  about?: ThingType[];

  /**
   * A a short description that summarizes a `CreativeWork`.
   */
  abstract?: Block[];

  /**
   * The authors of the `CreativeWork`.
   */
  authors?: Author[];

  /**
   * A secondary contributor to the `CreativeWork`.
   */
  contributors?: Author[];

  /**
   * People who edited the `CreativeWork`.
   */
  editors?: Person[];

  /**
   * The maintainers of the `CreativeWork`.
   */
  maintainers?: PersonOrOrganization[];

  /**
   * Comments about this creative work.
   */
  comments?: Comment[];

  /**
   * Date/time of creation.
   */
  dateCreated?: Date;

  /**
   * Date/time that work was received.
   */
  dateReceived?: Date;

  /**
   * Date/time of acceptance.
   */
  dateAccepted?: Date;

  /**
   * Date/time of most recent modification.
   */
  dateModified?: Date;

  /**
   * Date of first publication.
   */
  datePublished?: Date;

  /**
   * People or organizations that funded the `CreativeWork`.
   */
  funders?: PersonOrOrganization[];

  /**
   * Grants that funded the `CreativeWork`; reverse of `fundedItems`.
   */
  fundedBy?: GrantOrMonetaryGrant[];

  /**
   * Genre of the creative work, broadcast channel or group.
   */
  genre?: string[];

  /**
   * Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
   */
  keywords?: string[];

  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   */
  isPartOf?: CreativeWorkType;

  /**
   * License documents that applies to this content, typically indicated by URL.
   */
  licenses?: CreativeWorkTypeOrText[];

  /**
   * Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
   */
  parts?: CreativeWorkType[];

  /**
   * A publisher of the CreativeWork.
   */
  publisher?: PersonOrOrganization;

  /**
   * References to other creative works, such as another publication, web page, scholarly article, etc.
   */
  references?: CreativeWorkTypeOrText[];

  /**
   * The textual content of this creative work.
   */
  text?: Text;

  /**
   * The title of the creative work.
   */
  title?: Inline[];

  /**
   * The version of the creative work.
   */
  version?: StringOrNumber;

  constructor(options?: Partial<CreativeWork>) {
    super();
    this.type = "CreativeWork";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `CreativeWork`
*/
export function creativeWork(options?: Partial<CreativeWork>): CreativeWork {
  return new CreativeWork(options);
}
