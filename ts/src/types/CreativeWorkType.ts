// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Article } from "./Article.js";
import { type AudioObject } from "./AudioObject.js";
import { type Claim } from "./Claim.js";
import { type Collection } from "./Collection.js";
import { type Comment } from "./Comment.js";
import { type Datatable } from "./Datatable.js";
import { type Figure } from "./Figure.js";
import { type ImageObject } from "./ImageObject.js";
import { type MediaObject } from "./MediaObject.js";
import { type Periodical } from "./Periodical.js";
import { type PublicationIssue } from "./PublicationIssue.js";
import { type PublicationVolume } from "./PublicationVolume.js";
import { type Review } from "./Review.js";
import { type SoftwareApplication } from "./SoftwareApplication.js";
import { type SoftwareSourceCode } from "./SoftwareSourceCode.js";
import { type Table } from "./Table.js";
import { type VideoObject } from "./VideoObject.js";

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
  Figure |
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
    case "Figure":
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
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for CreativeWorkType: ${other.type}`);
  }
}
