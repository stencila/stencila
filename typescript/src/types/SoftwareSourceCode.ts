// Generated file. Do not edit; see `rust/schema-gen` crate.

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
import { String } from './String';
import { StringOrNumber } from './StringOrNumber';
import { ThingType } from './ThingType';

// Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
export class SoftwareSourceCode {
  // The type of this item
  type = "SoftwareSourceCode";

  // The identifier for this item
  id?: String;

  // Alternate names (aliases) for the item.
  alternateNames?: String[];

  // A description of the item.
  description?: Block[];

  // Any kind of identifier for any kind of Thing.
  identifiers?: PropertyValueOrString[];

  // Images of the item.
  images?: ImageObjectOrString[];

  // The name of the item.
  name?: String;

  // The URL of the item.
  url?: String;

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
  genre?: String[];

  // Keywords or tags used to describe this content.
  // Multiple entries in a keywords list are typically delimited by commas.
  keywords?: String[];

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
  text?: String;

  // The title of the creative work.
  title?: Inline[];

  // The version of the creative work.
  version?: StringOrNumber;

  // Link to the repository where the un-compiled, human readable code and related
  // code is located.
  codeRepository?: String;

  // What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
  codeSampleType?: String;

  // The computer programming language.
  programmingLanguage?: String;

  // Runtime platform or script interpreter dependencies (Example - Java v1,
  // Python2.3, .Net Framework 3.0).
  runtimePlatform?: String[];

  // Dependency requirements for the software.
  softwareRequirements?: SoftwareSourceCodeOrSoftwareApplicationOrString[];

  // Target operating system or product to which the code applies.
  targetProducts?: SoftwareApplication[];

  constructor(options?: SoftwareSourceCode) {
    if (options) Object.assign(this, options)
    
  }
}
