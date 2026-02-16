// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Agent } from "./Agent.js";
import { type Article } from "./Article.js";
import { type AudioObject } from "./AudioObject.js";
import { type Chat } from "./Chat.js";
import { type Claim } from "./Claim.js";
import { type Collection } from "./Collection.js";
import { type Comment } from "./Comment.js";
import { type Datatable } from "./Datatable.js";
import { type Figure } from "./Figure.js";
import { type File } from "./File.js";
import { type ImageObject } from "./ImageObject.js";
import { type MediaObject } from "./MediaObject.js";
import { type Periodical } from "./Periodical.js";
import { type Prompt } from "./Prompt.js";
import { type PublicationIssue } from "./PublicationIssue.js";
import { type PublicationVolume } from "./PublicationVolume.js";
import { type Review } from "./Review.js";
import { type Skill } from "./Skill.js";
import { type SoftwareApplication } from "./SoftwareApplication.js";
import { type SoftwareSourceCode } from "./SoftwareSourceCode.js";
import { type Table } from "./Table.js";
import { type VideoObject } from "./VideoObject.js";
import { type Workflow } from "./Workflow.js";

/**
 * Union type for all types that are descended from `CreativeWork`
 */
export type CreativeWorkVariant =
  Agent |
  Article |
  AudioObject |
  Chat |
  Claim |
  Collection |
  Comment |
  Datatable |
  Figure |
  File |
  ImageObject |
  MediaObject |
  Periodical |
  Prompt |
  PublicationIssue |
  PublicationVolume |
  Review |
  Skill |
  SoftwareApplication |
  SoftwareSourceCode |
  Table |
  VideoObject |
  Workflow;

/**
 * Create a `CreativeWorkVariant` from an object
 */
export function creativeWorkVariant(other: CreativeWorkVariant): CreativeWorkVariant {
  switch(other.type) {
    case "Agent":
    case "Article":
    case "AudioObject":
    case "Chat":
    case "Claim":
    case "Collection":
    case "Comment":
    case "Datatable":
    case "Figure":
    case "File":
    case "ImageObject":
    case "MediaObject":
    case "Periodical":
    case "Prompt":
    case "PublicationIssue":
    case "PublicationVolume":
    case "Review":
    case "Skill":
    case "SoftwareApplication":
    case "SoftwareSourceCode":
    case "Table":
    case "VideoObject":
    case "Workflow":
      return hydrate(other) as CreativeWorkVariant
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for CreativeWorkVariant: ${other.type}`);
  }
}
