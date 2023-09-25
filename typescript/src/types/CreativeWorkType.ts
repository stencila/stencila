// Generated file; do not edit. See `../rust/schema-gen` crate.
            
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
    case "Article": return Article.from(other as Article);
    case "AudioObject": return AudioObject.from(other as AudioObject);
    case "Claim": return Claim.from(other as Claim);
    case "Collection": return Collection.from(other as Collection);
    case "Comment": return Comment.from(other as Comment);
    case "Datatable": return Datatable.from(other as Datatable);
    case "Directory": return Directory.from(other as Directory);
    case "Figure": return Figure.from(other as Figure);
    case "File": return File.from(other as File);
    case "ImageObject": return ImageObject.from(other as ImageObject);
    case "MediaObject": return MediaObject.from(other as MediaObject);
    case "Periodical": return Periodical.from(other as Periodical);
    case "PublicationIssue": return PublicationIssue.from(other as PublicationIssue);
    case "PublicationVolume": return PublicationVolume.from(other as PublicationVolume);
    case "Review": return Review.from(other as Review);
    case "SoftwareApplication": return SoftwareApplication.from(other as SoftwareApplication);
    case "SoftwareSourceCode": return SoftwareSourceCode.from(other as SoftwareSourceCode);
    case "Table": return Table.from(other as Table);
    case "VideoObject": return VideoObject.from(other as VideoObject);
    default: throw new Error(`Unexpected type for CreativeWorkType: ${other.type}`);
  }
}
