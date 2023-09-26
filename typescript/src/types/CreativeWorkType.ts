// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { Article } from "./Article.js";
import { AudioObject } from "./AudioObject.js";
import { Claim } from "./Claim.js";
import { Collection } from "./Collection.js";
import { Comment } from "./Comment.js";
import { Datatable } from "./Datatable.js";
import { Directory } from "./Directory.js";
import { Figure } from "./Figure.js";
import { File } from "./File.js";
import { ImageObject } from "./ImageObject.js";
import { MediaObject } from "./MediaObject.js";
import { Periodical } from "./Periodical.js";
import { PublicationIssue } from "./PublicationIssue.js";
import { PublicationVolume } from "./PublicationVolume.js";
import { Review } from "./Review.js";
import { SoftwareApplication } from "./SoftwareApplication.js";
import { SoftwareSourceCode } from "./SoftwareSourceCode.js";
import { Table } from "./Table.js";
import { VideoObject } from "./VideoObject.js";

/**
 * Union type for all types that are descended from `CreativeWork`
 */
export type CreativeWorkType =
  Article |
  AudioObject |
  Claim |
  Collection |
  Comment |
  Datatable |
  Directory |
  Figure |
  File |
  ImageObject |
  MediaObject |
  Periodical |
  PublicationIssue |
  PublicationVolume |
  Review |
  SoftwareApplication |
  SoftwareSourceCode |
  Table |
  VideoObject;

/**
 * Create a `CreativeWorkType` from an object
 */
export function creativeWorkType(other: CreativeWorkType): CreativeWorkType {
  switch(other.type) {
    case "Article":
    case "AudioObject":
    case "Claim":
    case "Collection":
    case "Comment":
    case "Datatable":
    case "Directory":
    case "Figure":
    case "File":
    case "ImageObject":
    case "MediaObject":
    case "Periodical":
    case "PublicationIssue":
    case "PublicationVolume":
    case "Review":
    case "SoftwareApplication":
    case "SoftwareSourceCode":
    case "Table":
    case "VideoObject":
      return hydrate(other) as CreativeWorkType
    default:
      throw new Error(`Unexpected type for CreativeWorkType: ${other.type}`);
  }
}
