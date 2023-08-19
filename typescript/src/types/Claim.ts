// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { ClaimType } from './ClaimType';
import { Comment } from './Comment';
import { CreativeWorkType } from './CreativeWorkType';
import { CreativeWorkTypeOrString } from './CreativeWorkTypeOrString';
import { Date } from './Date';
import { GrantOrMonetaryGrant } from './GrantOrMonetaryGrant';
import { ImageObjectOrString } from './ImageObjectOrString';
import { Inline } from './Inline';
import { Person } from './Person';
import { PersonOrOrganization } from './PersonOrOrganization';
import { PropertyValueOrString } from './PropertyValueOrString';
import { StringOrNumber } from './StringOrNumber';
import { ThingType } from './ThingType';

// A claim represents specific reviewable facts or statements.
export class Claim {
  type = "Claim";

  // The identifier for this item
  id?: string;

  // Alternate names (aliases) for the item.
  alternateNames?: string[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name?: string;

  // The URL of the item.
  url?: string;

  // The subject matter of the content.
  about?: ThingType[];

  // The authors of this creative work.
  authors?: PersonOrOrganization[];

  // Comments about this creative work.
  comments?: Comment[];

  // Content of the claim, usually a single paragraph.
  content: Block[];

  // Date/time of creation.
  dateCreated?: Date;

  // Date/time that work was received.
  dateReceived?: Date;

  // Date/time of acceptance.
  dateAccepted?: Date;

  // Date/time of most recent modification.
  dateModified?: Date;

  // Date of first publication.
  datePublished?: Date;

  // People who edited the `CreativeWork`.
  editors?: Person[];

  // People or organizations that funded the `CreativeWork`.
  funders?: PersonOrOrganization[];

  // Grants that funded the `CreativeWork`; reverse of `fundedItems`.
  fundedBy?: GrantOrMonetaryGrant[];

  // Genre of the creative work, broadcast channel or group.
  genre?: string[];

  // Keywords or tags used to describe this content.
  // Multiple entries in a keywords list are typically delimited by commas.
  keywords?: string[];

  // An item or other CreativeWork that this CreativeWork is a part of.
  isPartOf?: CreativeWorkType;

  // License documents that applies to this content, typically indicated by URL.
  licenses?: CreativeWorkTypeOrString[];

  // The people or organizations who maintain this CreativeWork.
  maintainers?: PersonOrOrganization[];

  // Elements of the collection which can be a variety of different elements,
  // such as Articles, Datatables, Tables and more.
  parts?: CreativeWorkType[];

  // A publisher of the CreativeWork.
  publisher?: PersonOrOrganization;

  // References to other creative works, such as another publication,
  // web page, scholarly article, etc.
  references?: CreativeWorkTypeOrString[];

  // The textual content of this creative work.
  text?: string;

  // The title of the creative work.
  title?: Inline[];

  // The version of the creative work.
  version?: StringOrNumber;

  // The type of the claim.
  claimType: ClaimType;

  // A short label for the claim.
  label?: string;

  constructor(content: Block[], claimType: ClaimType, options?: Claim) {
    if (options) Object.assign(this, options)
    this.content = content;
    this.claimType = claimType;
  }
}
