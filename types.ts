/* eslint-disable */
/**
 * This file was automatically generated.
 * Do not modify it by hand. Instead, modify the source `.schema.yaml` file
 * in the `schema` directory and run `npm run build` to regenerate this file.
 */

/**
 * Describes a valid value for any node in the tree.
 *
 */
export type Node =
  | null
  | boolean
  | number
  | string
  | any[]
  | {
      [k: string]: any;
    }
  | Thing;
/**
 * Block content.
 */
export type BlockContent =
  | CodeBlock
  | CodeChunk
  | Heading
  | List
  | ListItem
  | Paragraph
  | QuoteBlock
  | Table
  | ThematicBreak;
/**
 * Inline content.
 */
export type InlineContent =
  | null
  | boolean
  | number
  | string
  | Emphasis
  | Strong
  | Delete
  | Subscript
  | Superscript
  | Quote
  | Code
  | CodeExpr
  | Link
  | ImageObject;
/**
 * A directive to include content from an external source (e.g. file, URL) or content.
 *
 */
export type Include = {
  [k: string]: any;
};

export interface Types {
  Article: Article;
  AudioObject: AudioObject;
  BlockContent: BlockContent;
  Brand: Brand;
  Code: Code;
  CodeBlock: CodeBlock;
  CodeChunk: CodeChunk;
  CodeExpr: CodeExpr;
  Collection: Collection;
  ContactPoint: ContactPoint;
  CreativeWork: CreativeWork;
  Datatable: Datatable;
  DatatableColumn: DatatableColumn;
  DatatableColumnSchema: DatatableColumnSchema;
  Delete: Delete;
  Emphasis: Emphasis;
  Entity: Entity;
  Environment: Environment;
  Heading: Heading;
  ImageObject: ImageObject;
  Include: Include;
  InlineContent: InlineContent;
  Link: Link;
  List: List;
  ListItem: ListItem;
  Mark: Mark;
  MediaObject: MediaObject;
  Mount: Mount;
  Node: Node;
  Organization: Organization;
  Paragraph: Paragraph;
  Person: Person;
  Product: Product;
  Quote: Quote;
  QuoteBlock: QuoteBlock;
  ResourceParameters: ResourceParameters;
  SoftwareApplication: SoftwareApplication;
  SoftwareSession: SoftwareSession;
  SoftwareSourceCode: SoftwareSourceCode;
  Strong: Strong;
  Subscript: Subscript;
  Superscript: Superscript;
  Table: Table;
  TableCell: TableCell;
  TableRow: TableRow;
  ThematicBreak: ThematicBreak;
  Thing: Thing;
  VideoObject: VideoObject;
  [k: string]: any;
}
export interface Article {
  /**
   * The name of the type and all descendant types.
   */
  type: "Article";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title: string;
  version?: string | number;
  /**
   * The computational environment in which the document should be executed.
   *
   */
  environment?: Environment;
}
/**
 * A person (alive, dead, undead, or fictional). https://schema.org/Person.
 */
export interface Person {
  /**
   * The name of the type and all descendant types.
   */
  type: "Person";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * Postal address for the person.
   */
  address?: string;
  /**
   * Organizations that the person is affiliated with.
   */
  affiliations?: Organization[];
  /**
   * Email addresses for the person.
   */
  emails?: string[];
  /**
   * Family name. In the U.S., the last name of an Person.
   * This can be used along with givenName instead of the name property.
   *
   */
  familyNames?: {
    [k: string]: any;
  } & string[];
  /**
   * A person or organization that supports (sponsors) something through
   * some kind of financial contribution.
   *
   */
  funders?: (Organization | Person)[];
  /**
   * Given name. In the U.S., the first name of a Person.
   * This can be used along with familyName instead of the name property.
   *
   */
  givenNames?: {
    [k: string]: any;
  } & string[];
  /**
   * An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
   */
  honorificPrefix?: string;
  /**
   * An honorific suffix after a person's name such as MD/PhD/MSCSW.
   */
  honorificSuffix?: string;
  /**
   * The job title of the person (for example, Financial Manager).
   */
  jobTitle?: string;
  /**
   * An organization (or program membership) to which this person belongs.
   */
  memberOf?: [Organization];
  /**
   * Telephone numbers for the person.
   */
  telephoneNumbers?: string[];
}
/**
 * An organization such as a school, NGO, corporation, club, etc. https://schema.org/Organization.
 */
export interface Organization {
  /**
   * The name of the type and all descendant types.
   */
  type: "Organization";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * Postal address for the organization.
   *
   */
  address?: string;
  /**
   * Brands that the organization is connected with.
   *
   */
  brands?: Brand[];
  /**
   * Correspondence/Contact points for the organization.
   *
   */
  contactPoints?: ContactPoint[];
  /**
   * Departments within the organization. For example, Department of Computer Science, Research & Development etc.
   *
   */
  departments?: Organization[];
  /**
   * Organization(s) or person(s) funding the organization.
   *
   */
  funders?: (Organization | Person)[];
  /**
   * Legal name for the Organization. Should only include letters and spaces.
   *
   */
  legalName?: string;
  /**
   * Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
   *
   */
  parentOrganization?: Organization;
}
/**
 * A brand is a name used by an organization or business person for labeling a product,
 * product group, or similar. https://schema.org/Brand.
 *
 */
export interface Brand {
  /**
   * The name of the type and all descendant types.
   */
  type: "Brand";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * A logo of of the brand. It can be either a URL of the image or image itself.
   *
   */
  logo?: string | ImageObject;
  /**
   * Short reviews of the brand and/or the products it represents.
   *
   */
  reviews?: string[];
}
/**
 * An image file. https://schema.org/ImageObject
 *
 */
export interface ImageObject {
  /**
   * The name of the type and all descendant types.
   */
  type: "ImageObject";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
   *
   */
  bitrate?: number;
  /**
   * File size in megabits (Mbit, Mb).
   *
   */
  contentSize?: number;
  /**
   * URL for the actual bytes of the media object, for example the image file or video file.
   *
   */
  contentUrl: string;
  /**
   * URL that can be used to embed the media on a web page via a specific media player.
   *
   */
  embedUrl?: string;
  /**
   * Media type (MIME type) as per http://www.iana.org/assignments/media-types/media-types.xhtml.
   *
   */
  format?: string;
  /**
   * The caption for this image.
   *
   */
  caption?: string;
  /**
   * Thumbnail image of this image.
   *
   */
  thumbnail?: ImageObject;
}
/**
 * The most generic kind of creative work, including books, movies, photographs,
 * software programs, etc. https://schema.org/CreativeWork
 *
 */
export interface CreativeWork {
  /**
   * The name of the type and all descendant types.
   */
  type:
    | "CreativeWork"
    | "Article"
    | "AudioObject"
    | "CodeChunk"
    | "CodeExpr"
    | "Collection"
    | "Datatable"
    | "ImageObject"
    | "MediaObject"
    | "SoftwareApplication"
    | "SoftwareSourceCode"
    | "Table"
    | "VideoObject";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
}
/**
 * The most generic type of item https://schema.org/Thing.
 */
export interface Thing {
  /**
   * The name of the type and all descendant types.
   */
  type:
    | "Thing"
    | "Article"
    | "AudioObject"
    | "Brand"
    | "Code"
    | "CodeBlock"
    | "CodeChunk"
    | "CodeExpr"
    | "Collection"
    | "ContactPoint"
    | "CreativeWork"
    | "Datatable"
    | "DatatableColumn"
    | "Environment"
    | "ImageObject"
    | "MediaObject"
    | "Mount"
    | "Organization"
    | "Person"
    | "Product"
    | "ResourceParameters"
    | "SoftwareApplication"
    | "SoftwareSession"
    | "SoftwareSourceCode"
    | "Table"
    | "VideoObject";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
}
/**
 * A contact point—for example, a R&D department. https://schema.org/ContactPoint.
 */
export interface ContactPoint {
  /**
   * The name of the type and all descendant types.
   */
  type: "ContactPoint";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * Languages (human not programming) in which it is possible to communicate with the organization/department etc.
   *
   */
  availableLanguages?: string[];
  /**
   * Email address for correspondence. It must be provided in a valid email format (eg. info@example.com ).
   *
   */
  emails?: string[];
  /**
   * "Phone contact number. Accepted formats: +44 123455, (02)12345, 006645667."
   *
   */
  telephone?: string;
}
/**
 * A computational environment.
 */
export interface Environment {
  /**
   * The name of the type and all descendant types.
   */
  type: "Environment";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * Other environments that this environment extends by adding or removing packages.,
   */
  extends?: Environment[];
  /**
   * The packages that this environment adds to the base environments listed under `extends` (if any).,
   */
  adds?: SoftwareSourceCode[];
  /**
   * The packages that this environment removes from the base environments listed under `extends` (if any).,
   */
  removes?: SoftwareSourceCode[];
  /**
   * Source of environment definition. For example, a URL to a Dockerfile, a path to Singularity recipe file,
   * or a path to a filesystem folder.,
   *
   */
  environmentSource?: string;
}
/**
 * Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
 *
 */
export interface SoftwareSourceCode {
  /**
   * The name of the type and all descendant types.
   */
  type: "SoftwareSourceCode" | "CodeChunk" | "CodeExpr";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Link to the repository where the un-compiled, human readable code and related code is located (SVN, github, CodePlex)
   *
   */
  codeRepository?: string;
  /**
   * What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
   *
   */
  codeSampleType?: string;
  /**
   * The people or organizations who maintain the software.
   *
   */
  maintainers?: [Organization, Person];
  /**
   * The computer programming language.
   *
   */
  programmingLanguage?: string;
  /**
   * Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0).
   *
   */
  runtimePlatform?: [string];
  /**
   * Component dependency requirements for application.
   * This includes runtime environments and shared libraries that
   * are not included in the application distribution package, but
   * required to run the application (Examples include DirectX, Java or .NET runtime).
   *
   */
  softwareRequirements?: (SoftwareSourceCode | SoftwareApplication | string)[];
  /**
   * Target Operating System / Product to which the code applies.
   * If applies to several versions, just the product name can be used.
   *
   */
  targetProducts?: [SoftwareApplication];
}
/**
 * A software application.
 *
 */
export interface SoftwareApplication {
  /**
   * The name of the type and all descendant types.
   */
  type: "SoftwareApplication";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Component dependency requirements for application. This includes runtime environments and shared libraries that are not included in the application distribution package, but required to run the application (Examples: DirectX, Java or .NET runtime).
   *
   */
  softwareRequirements?: [SoftwareApplication];
  /**
   * Version of the software instance.
   */
  softwareVersion?: string;
}
/**
 * An audio file. https://schema.org/AudioObject
 *
 */
export interface AudioObject {
  /**
   * The name of the type and all descendant types.
   */
  type: "AudioObject";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
   *
   */
  bitrate?: number;
  /**
   * File size in megabits (Mbit, Mb).
   *
   */
  contentSize?: number;
  /**
   * URL for the actual bytes of the media object, for example the image file or video file.
   *
   */
  contentUrl: string;
  /**
   * URL that can be used to embed the media on a web page via a specific media player.
   *
   */
  embedUrl?: string;
  /**
   * Media type (MIME type) as per http://www.iana.org/assignments/media-types/media-types.xhtml.
   *
   */
  format?: string;
  /**
   * The caption for this audio recording.
   *
   */
  caption?: string;
  /**
   * The transcript of this audio recording.
   *
   */
  transcript?: string;
}
/**
 * A code block.
 */
export interface CodeBlock {
  /**
   * The name of the type and all descendant types.
   */
  type: "CodeBlock";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The programming langauge of the code.
   */
  language?: string;
  /**
   * The text value.
   */
  value: string;
}
/**
 * A executable chunk of code.
 *
 */
export interface CodeChunk {
  /**
   * The name of the type and all descendant types.
   */
  type: "CodeChunk";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Link to the repository where the un-compiled, human readable code and related code is located (SVN, github, CodePlex)
   *
   */
  codeRepository?: string;
  /**
   * What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
   *
   */
  codeSampleType?: string;
  /**
   * The people or organizations who maintain the software.
   *
   */
  maintainers?: [Organization, Person];
  /**
   * The computer programming language.
   *
   */
  programmingLanguage?: string;
  /**
   * Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0).
   *
   */
  runtimePlatform?: [string];
  /**
   * Component dependency requirements for application.
   * This includes runtime environments and shared libraries that
   * are not included in the application distribution package, but
   * required to run the application (Examples include DirectX, Java or .NET runtime).
   *
   */
  softwareRequirements?: (SoftwareSourceCode | SoftwareApplication | string)[];
  /**
   * Target Operating System / Product to which the code applies.
   * If applies to several versions, just the product name can be used.
   *
   */
  targetProducts?: [SoftwareApplication];
  /**
   * Outputs from executing the chunk.
   *
   */
  outputs?: Node[];
}
/**
 * Heading
 */
export interface Heading {
  /**
   * The name of the type and all descendant types.
   */
  type: "Heading";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  depth: number;
  /**
   * Content of the heading.
   */
  content: InlineContent[];
}
/**
 * Emphasised content.
 */
export interface Emphasis {
  /**
   * The name of the type and all descendant types.
   */
  type: "Emphasis";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The content that is marked.
   *
   */
  content: InlineContent[];
}
/**
 * Strongly emphasised content. Analagous to,
 *   - JATS [`<bold>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/bold.html)
 *   - HTML [`<strong>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong)
 *   - MDAST [`Strong`](https://github.com/syntax-tree/mdast#strong)
 *   - Pandoc [`Strong`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L257)
 *
 */
export interface Strong {
  /**
   * The name of the type and all descendant types.
   */
  type: "Strong";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The content that is marked.
   *
   */
  content: InlineContent[];
}
/**
 * Content that is marked for deletion
 */
export interface Delete {
  /**
   * The name of the type and all descendant types.
   */
  type: "Delete";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The content that is marked.
   *
   */
  content: InlineContent[];
}
/**
 * Subscripted content. Analagous to,
 *   - JATS [`<sub>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/sub.html)
 *   - HTML [`<sub>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub)
 *   - Pandoc [`Subscript`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L260)
 *
 */
export interface Subscript {
  /**
   * The name of the type and all descendant types.
   */
  type: "Subscript";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The content that is marked.
   *
   */
  content: InlineContent[];
}
/**
 * Superscripted content. Analagous to,
 *   - JATS [`<sup>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/sup.html)
 *   - HTML [`<sup>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup)
 *   - Pandoc [`Superscript`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L259)
 *
 */
export interface Superscript {
  /**
   * The name of the type and all descendant types.
   */
  type: "Superscript";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The content that is marked.
   *
   */
  content: InlineContent[];
}
/**
 * Inline, quoted content. Analagous to,
 *   - HTML [`<q>` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)
 *   - Pandoc [`Quoted`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L262)
 *
 */
export interface Quote {
  /**
   * The name of the type and all descendant types.
   */
  type: "Quote";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The content that is marked.
   *
   */
  content: InlineContent[];
  /**
   * The source of the quote.
   */
  citation?: string;
}
/**
 * Inline code.
 */
export interface Code {
  /**
   * The name of the type and all descendant types.
   */
  type: "Code" | "CodeBlock";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The programming langauge of the code.
   */
  language?: string;
  /**
   * The text value.
   */
  value: string;
}
/**
 * An expression.
 */
export interface CodeExpr {
  /**
   * The name of the type and all descendant types.
   */
  type: "CodeExpr";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Link to the repository where the un-compiled, human readable code and related code is located (SVN, github, CodePlex)
   *
   */
  codeRepository?: string;
  /**
   * What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
   *
   */
  codeSampleType?: string;
  /**
   * The people or organizations who maintain the software.
   *
   */
  maintainers?: [Organization, Person];
  /**
   * The computer programming language.
   *
   */
  programmingLanguage?: string;
  /**
   * Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0).
   *
   */
  runtimePlatform?: [string];
  /**
   * Component dependency requirements for application.
   * This includes runtime environments and shared libraries that
   * are not included in the application distribution package, but
   * required to run the application (Examples include DirectX, Java or .NET runtime).
   *
   */
  softwareRequirements?: (SoftwareSourceCode | SoftwareApplication | string)[];
  /**
   * Target Operating System / Product to which the code applies.
   * If applies to several versions, just the product name can be used.
   *
   */
  targetProducts?: [SoftwareApplication];
  value?: Node;
}
/**
 * A hyperlink to other pages, sections within the same document, resources, or any URL.
 */
export interface Link {
  /**
   * The name of the type and all descendant types.
   */
  type: "Link";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  content: InlineContent[];
  target: string;
  /**
   * A title for the link.
   */
  title?: string;
  /**
   * The relation between the target and the current thing.
   */
  relation?: string;
}
/**
 * A list of items.
 *
 */
export interface List {
  /**
   * The name of the type and all descendant types.
   */
  type: "List";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The items in the list
   */
  items: ListItem[];
  /**
   * Type of ordering.
   */
  order?: "ascending" | "descending" | "unordered";
}
/**
 * A single item in a list.
 */
export interface ListItem {
  /**
   * The name of the type and all descendant types.
   */
  type: "ListItem";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  content: Node[];
  checked?: boolean;
}
/**
 * Paragraph
 */
export interface Paragraph {
  /**
   * The name of the type and all descendant types.
   */
  type: "Paragraph";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  content: InlineContent[];
}
/**
 * A section quoted from somewhere else.
 *
 */
export interface QuoteBlock {
  /**
   * The name of the type and all descendant types.
   */
  type: "QuoteBlock";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The source of the quote
   */
  citation?: string;
  content: BlockContent[];
}
/**
 * A table.
 */
export interface Table {
  /**
   * The name of the type and all descendant types.
   */
  type: "Table";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Rows of cells in the table.
   *
   */
  rows: TableRow[];
}
/**
 * A row within a Table.
 */
export interface TableRow {
  /**
   * The name of the type and all descendant types.
   */
  type: "TableRow";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * An array of cells in the row.
   */
  cells: TableCell[];
}
/**
 * A cell within a `Table`.
 *
 */
export interface TableCell {
  /**
   * The name of the type and all descendant types.
   */
  type: "TableCell";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The name of the cell. Cell's have an implicit name derived from their position in the table
   * e.g. `C4` for the cell in the third column and fourth row. However this name can be overridden
   * with an explicit name, e.g. `rate`.
   *
   */
  name?: string;
  /**
   * How many columns the cell extends.
   *
   */
  colspan?: number;
  /**
   * How many columns the cell extends.
   *
   */
  rowspan?: number;
  /**
   * Contents of the table cell.
   *
   */
  content: InlineContent[];
}
/**
 * A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
 *
 */
export interface ThematicBreak {
  /**
   * The name of the type and all descendant types.
   */
  type: "ThematicBreak";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
}
/**
 * A created collection of CreativeWorks or other artefacts.
 *
 */
export interface Collection {
  /**
   * The name of the type and all descendant types.
   */
  type: "Collection";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
}
/**
 * A table of data.
 */
export interface Datatable {
  /**
   * The name of the type and all descendant types.
   */
  type: "Datatable";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * TODO
   */
  columns: DatatableColumn[];
}
export interface DatatableColumn {
  /**
   * The name of the type and all descendant types.
   */
  type: "DatatableColumn";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata for the column.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The schema to use to validate data in the column.
   */
  schema?: DatatableColumnSchema;
  /**
   * The values of the column.
   */
  values: any[];
}
export interface DatatableColumnSchema {
  /**
   * The name of the type and all descendant types.
   */
  type: "DatatableColumnSchema";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  items: {
    [k: string]: any;
  };
  uniqueItems?: boolean;
}
/**
 * The most basic item, defining the minimum properties required.
 */
export interface Entity {
  /**
   * The name of the type and all descendant types.
   */
  type:
    | "Entity"
    | "Article"
    | "AudioObject"
    | "Brand"
    | "Code"
    | "CodeBlock"
    | "CodeChunk"
    | "CodeExpr"
    | "Collection"
    | "ContactPoint"
    | "CreativeWork"
    | "Datatable"
    | "DatatableColumn"
    | "DatatableColumnSchema"
    | "Delete"
    | "Emphasis"
    | "Environment"
    | "Heading"
    | "ImageObject"
    | "Include"
    | "Link"
    | "List"
    | "ListItem"
    | "Mark"
    | "MediaObject"
    | "Mount"
    | "Organization"
    | "Paragraph"
    | "Person"
    | "Product"
    | "Quote"
    | "QuoteBlock"
    | "ResourceParameters"
    | "SoftwareApplication"
    | "SoftwareSession"
    | "SoftwareSourceCode"
    | "Strong"
    | "Subscript"
    | "Superscript"
    | "Table"
    | "TableCell"
    | "TableRow"
    | "ThematicBreak"
    | "Thing"
    | "VideoObject";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
}
/**
 * A base class for nodes that mark some other inline content (e.g. `string` or other `InlineContent` nodes) in some way (e.g. as being emphasised, or quoted).
 *
 */
export interface Mark {
  /**
   * The name of the type and all descendant types.
   */
  type: "Mark" | "Delete" | "Emphasis" | "Quote" | "Strong" | "Subscript" | "Superscript";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * The content that is marked.
   *
   */
  content: InlineContent[];
}
/**
 * A media object, such as an image, video, or audio object embedded in a web page or a
 * downloadable dataset. https://schema.org/MediaObject
 *
 */
export interface MediaObject {
  /**
   * The name of the type and all descendant types.
   */
  type: "MediaObject" | "AudioObject" | "ImageObject" | "VideoObject";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
   *
   */
  bitrate?: number;
  /**
   * File size in megabits (Mbit, Mb).
   *
   */
  contentSize?: number;
  /**
   * URL for the actual bytes of the media object, for example the image file or video file.
   *
   */
  contentUrl: string;
  /**
   * URL that can be used to embed the media on a web page via a specific media player.
   *
   */
  embedUrl?: string;
  /**
   * Media type (MIME type) as per http://www.iana.org/assignments/media-types/media-types.xhtml.
   *
   */
  format?: string;
}
/**
 * Describes a volume mount from a host to container.
 *
 */
export interface Mount {
  /**
   * The name of the type and all descendant types.
   */
  type: "Mount";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The mount source directory on the host.
   */
  mountSource?: string;
  /**
   * The mount location inside the container.
   */
  mountDestination: string;
  /**
   * A list of options to use when applying the mount.
   */
  mountOptions?: [string];
  /**
   * The type of mount.
   */
  mountType?: string;
}
/**
 * Any offered product or service. For example, a pair of shoes; a concert ticket; the rental of a car;
 * a haircut; or an episode of a TV show streamed online. https://schema.org/Product
 *
 */
export interface Product {
  /**
   * The name of the type and all descendant types.
   */
  type: "Product";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  brand?: Brand1;
  /**
   * A logo of of the product. It can be either a URL of the image or image itself.
   *
   */
  logo?: string | ImageObject;
  /**
   * Product identification code.
   *
   */
  productID?: string;
}
/**
 * Brand that the product is labelled with.
 *
 */
export interface Brand1 {
  /**
   * The name of the type and all descendant types.
   */
  type: "Brand";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * A logo of of the brand. It can be either a URL of the image or image itself.
   *
   */
  logo?: string | ImageObject;
  /**
   * Short reviews of the brand and/or the products it represents.
   *
   */
  reviews?: string[];
}
/**
 * Describes limits or requested amounts for a particular resource (e.g. memory or CPU).
 *
 */
export interface ResourceParameters {
  /**
   * The name of the type and all descendant types.
   */
  type: "ResourceParameters";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The maximum amount of the resource that can be used.
   */
  resourceLimit?: number;
  /**
   * The amount of the resource that has been requested (and possibly reserved).
   */
  resourceRequested?: number;
}
/**
 * Represents a runtime session with the resources and image that is required by software to execute.
 *
 */
export interface SoftwareSession {
  /**
   * The name of the type and all descendant types.
   */
  type: "SoftwareSession";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * Volumes to mount in the session.
   */
  volumeMounts?: [Mount];
  cpuResource?: ResourceParameters;
  memoryResource?: ResourceParameters;
  /**
   * Definition of the environment to execute this session in.
   */
  environment: Environment;
}
/**
 * A video file. https://schema.org/VideoObject
 *
 */
export interface VideoObject {
  /**
   * The name of the type and all descendant types.
   */
  type: "VideoObject";
  /**
   * The identifier for this item.
   */
  id?: string;
  /**
   * Metadata associated with this item.
   */
  meta?: {
    [k: string]: any;
  };
  /**
   * Alternate names (aliases) for the item.
   */
  alternateNames?: string[];
  /**
   * A description of the item.
   */
  description?: string;
  /**
   * The name of the item.
   */
  name?: string;
  /**
   * The URL of the item.
   */
  url?: string;
  /**
   * The authors of this this creative work.
   */
  authors?: (Person | Organization)[];
  /**
   * Citations or references to other creative works, such as another publication,
   * web page, scholarly article, etc.
   *
   */
  citations?: (string | CreativeWork)[];
  /**
   * The structured content of this creative work c.f. property `text`.
   */
  content?: Node[];
  /**
   * Date/time of creation.
   */
  dateCreated?: string;
  /**
   * Date/time of most recent modification.
   */
  dateModified?: string;
  /**
   * Date of first publication.
   */
  datePublished?: string;
  /**
   * Persons who edited the CreativeWork.
   *
   */
  editors?: Person[];
  /**
   * Person or organisation that funded the CreativeWork.
   *
   */
  funders?: (Person | Organization)[];
  /**
   * An item or other CreativeWork that this CreativeWork is a part of.
   *
   */
  isPartOf?: CreativeWork;
  /**
   * License documents that applies to this content, typically indicated by URL.
   *
   */
  licenses?: (string | CreativeWork)[];
  /**
   * Elements of the collection which can be a variety of different elements,
   * such as Articles, Datatables, Tables and more.
   *
   */
  parts?: CreativeWork[];
  /**
   * A publisher of the CreativeWork.
   *
   */
  publisher?: Person | Organization;
  /**
   * The textual content of this creative work.
   */
  text?: string;
  title?: string;
  version?: string | number;
  /**
   * Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
   *
   */
  bitrate?: number;
  /**
   * File size in megabits (Mbit, Mb).
   *
   */
  contentSize?: number;
  /**
   * URL for the actual bytes of the media object, for example the image file or video file.
   *
   */
  contentUrl: string;
  /**
   * URL that can be used to embed the media on a web page via a specific media player.
   *
   */
  embedUrl?: string;
  /**
   * Media type (MIME type) as per http://www.iana.org/assignments/media-types/media-types.xhtml.
   *
   */
  format?: string;
  /**
   * The caption for this video recording.
   *
   */
  caption?: string;
  /**
   * Thumbnail image of this video recording.
   *
   */
  thumbnail?: ImageObject;
  /**
   * The transcript of this video recording.
   *
   */
  transcript?: string;
}
