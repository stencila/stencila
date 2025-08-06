// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { CreativeWorkType } from "./CreativeWorkType.js";
import { Date } from "./Date.js";
import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";
import { IntegerOrString } from "./IntegerOrString.js";
import { Person } from "./Person.js";
import { PersonOrOrganization } from "./PersonOrOrganization.js";
import { PropertyValueOrString } from "./PropertyValueOrString.js";
import { StringOrNumber } from "./StringOrNumber.js";

/**
 * A reference to a creative work, including books, movies, photographs, software programs, etc.
 */
export class Reference extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Reference";

  /**
   * The type of `CreativeWork` being referenced(e.g. Article, Book, Dataset).
   */
  workType?: CreativeWorkType;

  /**
   * The Digital Object Identifier (https://doi.org/) of the work being referenced.
   */
  doi?: string;

  /**
   * The authors of the work.
   */
  authors?: Author[];

  /**
   * People who edited the referenced work.
   */
  editors?: Person[];

  /**
   * A publisher of the referenced work.
   */
  publisher?: PersonOrOrganization;

  /**
   * Date of first publication.
   */
  date?: Date;

  /**
   * The title of the referenced work.
   */
  title?: Inline[];

  /**
   * Another `Reference` that this reference is a part of.
   */
  isPartOf?: Reference;

  /**
   * Identifies the volume of publication or multi-part work; for example, "iii" or "2".
   */
  volumeNumber?: IntegerOrString;

  /**
   * Identifies the issue of a serial publication; for example, "3" or "12".
   */
  issueNumber?: IntegerOrString;

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
   * The version/edition of the referenced work.
   */
  version?: StringOrNumber;

  /**
   * Any kind of identifier for the referenced work.
   */
  identifiers?: PropertyValueOrString[];

  /**
   * The URL of the referenced work.
   */
  url?: string;

  constructor(options?: Partial<Reference>) {
    super();
    this.type = "Reference";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Reference`
*/
export function reference(options?: Partial<Reference>): Reference {
  return new Reference(options);
}
