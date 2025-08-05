// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Block } from "./Block.js";
import { Comment } from "./Comment.js";
import { CreativeWorkType } from "./CreativeWorkType.js";
import { CreativeWorkVariant } from "./CreativeWorkVariant.js";
import { CreativeWorkVariantOrString } from "./CreativeWorkVariantOrString.js";
import { Date } from "./Date.js";
import { GrantOrMonetaryGrant } from "./GrantOrMonetaryGrant.js";
import { Inline } from "./Inline.js";
import { Person } from "./Person.js";
import { PersonOrOrganization } from "./PersonOrOrganization.js";
import { ProvenanceCount } from "./ProvenanceCount.js";
import { Reference } from "./Reference.js";
import { StringOrNumber } from "./StringOrNumber.js";
import { Text } from "./Text.js";
import { Thing } from "./Thing.js";
import { ThingVariant } from "./ThingVariant.js";

/**
 * A creative work, including books, movies, photographs, software programs, etc.
 */
export class CreativeWork extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CreativeWork";

  /**
   * The type of `CreativeWork` (e.g. article, book, software application).
   */
  workType?: CreativeWorkType;

  /**
   * The work's Digital Object Identifier (https://doi.org/).
   */
  doi?: string;

  /**
   * The subject matter of the content.
   */
  about?: ThingVariant[];

  /**
   * A short description that summarizes a `CreativeWork`.
   */
  abstract?: Block[];

  /**
   * The authors of the `CreativeWork`.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the content within the work.
   */
  provenance?: ProvenanceCount[];

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
  isPartOf?: CreativeWorkVariant;

  /**
   * License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.
   */
  licenses?: CreativeWorkVariantOrString[];

  /**
   * Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
   */
  parts?: CreativeWorkVariant[];

  /**
   * A publisher of the CreativeWork.
   */
  publisher?: PersonOrOrganization;

  /**
   * References to other creative works, such as another publication, web page, scholarly article, etc.
   */
  references?: Reference[];

  /**
   * The textual content of this creative work.
   */
  text?: Text;

  /**
   * The title of the creative work.
   */
  title?: Inline[];

  /**
   * URL of the repository where the un-compiled, human readable source of the work is located.
   */
  repository?: string;

  /**
   * The file system path of the source of the work.
   */
  path?: string;

  /**
   * The commit hash (or similar) of the source of the work.
   */
  commit?: string;

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
