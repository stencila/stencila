// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
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
import { SoftwareApplication } from './SoftwareApplication';
import { SoftwareSourceCodeOrSoftwareApplicationOrString } from './SoftwareSourceCodeOrSoftwareApplicationOrString';
import { StringOrNumber } from './StringOrNumber';
import { ThingType } from './ThingType';

// Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
export class SoftwareSourceCode {
  type = "SoftwareSourceCode";

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

  // The structured content of this creative work c.f. property `text`.
  content?: Block[];

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

  // Link to the repository where the un-compiled, human readable code and related
  // code is located.
  codeRepository?: string;

  // What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
  codeSampleType?: string;

  // The computer programming language.
  programmingLanguage?: string;

  // Runtime platform or script interpreter dependencies (Example - Java v1,
  // Python2.3, .Net Framework 3.0).
  runtimePlatform?: string[];

  // Dependency requirements for the software.
  softwareRequirements?: SoftwareSourceCodeOrSoftwareApplicationOrString[];

  // Target operating system or product to which the code applies.
  targetProducts?: SoftwareApplication[];

  constructor(options?: SoftwareSourceCode) {
    if (options) Object.assign(this, options)
    
  }
}
