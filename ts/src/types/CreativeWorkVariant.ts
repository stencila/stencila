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
import { type Evidence } from "./Evidence.js";
import { type Figure } from "./Figure.js";
import { type File } from "./File.js";
import { type Graph } from "./Graph.js";
import { type ImageObject } from "./ImageObject.js";
import { type MediaObject } from "./MediaObject.js";
import { type Periodical } from "./Periodical.js";
import { type Prompt } from "./Prompt.js";
import { type Protocol } from "./Protocol.js";
import { type PublicationIssue } from "./PublicationIssue.js";
import { type PublicationVolume } from "./PublicationVolume.js";
import { type Question } from "./Question.js";
import { type Request } from "./Request.js";
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
  Evidence |
  Figure |
  File |
  Graph |
  ImageObject |
  MediaObject |
  Periodical |
  Prompt |
  Protocol |
  PublicationIssue |
  PublicationVolume |
  Question |
  Request |
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
    case "Evidence":
    case "Figure":
    case "File":
    case "Graph":
    case "ImageObject":
    case "MediaObject":
    case "Periodical":
    case "Prompt":
    case "Protocol":
    case "PublicationIssue":
    case "PublicationVolume":
    case "Question":
    case "Request":
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
