// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Admonition } from "./Admonition.js";
import { type Array } from "./Array.js";
import { type ArrayHint } from "./ArrayHint.js";
import { type ArrayValidator } from "./ArrayValidator.js";
import { type Article } from "./Article.js";
import { type AudioObject } from "./AudioObject.js";
import { type AuthorRole } from "./AuthorRole.js";
import { type BooleanValidator } from "./BooleanValidator.js";
import { type Brand } from "./Brand.js";
import { type Button } from "./Button.js";
import { type CallArgument } from "./CallArgument.js";
import { type CallBlock } from "./CallBlock.js";
import { type Chat } from "./Chat.js";
import { type ChatMessage } from "./ChatMessage.js";
import { type ChatMessageGroup } from "./ChatMessageGroup.js";
import { type Cite } from "./Cite.js";
import { type CiteGroup } from "./CiteGroup.js";
import { type Claim } from "./Claim.js";
import { type CodeBlock } from "./CodeBlock.js";
import { type CodeChunk } from "./CodeChunk.js";
import { type CodeExpression } from "./CodeExpression.js";
import { type CodeInline } from "./CodeInline.js";
import { type CodeLocation } from "./CodeLocation.js";
import { type Collection } from "./Collection.js";
import { type Comment } from "./Comment.js";
import { type CompilationDigest } from "./CompilationDigest.js";
import { type CompilationMessage } from "./CompilationMessage.js";
import { type Config } from "./Config.js";
import { type ConstantValidator } from "./ConstantValidator.js";
import { type ContactPoint } from "./ContactPoint.js";
import { type Cord } from "./Cord.js";
import { type CreativeWork } from "./CreativeWork.js";
import { type Datatable } from "./Datatable.js";
import { type DatatableColumn } from "./DatatableColumn.js";
import { type DatatableColumnHint } from "./DatatableColumnHint.js";
import { type DatatableHint } from "./DatatableHint.js";
import { type Date } from "./Date.js";
import { type DateTime } from "./DateTime.js";
import { type DateTimeValidator } from "./DateTimeValidator.js";
import { type DateValidator } from "./DateValidator.js";
import { type DefinedTerm } from "./DefinedTerm.js";
import { type DeleteBlock } from "./DeleteBlock.js";
import { type DeleteInline } from "./DeleteInline.js";
import { type Directory } from "./Directory.js";
import { type Duration } from "./Duration.js";
import { type DurationValidator } from "./DurationValidator.js";
import { type Emphasis } from "./Emphasis.js";
import { type EnumValidator } from "./EnumValidator.js";
import { type Enumeration } from "./Enumeration.js";
import { type ExecutionDependant } from "./ExecutionDependant.js";
import { type ExecutionDependency } from "./ExecutionDependency.js";
import { type ExecutionMessage } from "./ExecutionMessage.js";
import { type ExecutionTag } from "./ExecutionTag.js";
import { type Figure } from "./Figure.js";
import { type File } from "./File.js";
import { type ForBlock } from "./ForBlock.js";
import { type Form } from "./Form.js";
import { type Function } from "./Function.js";
import { type Grant } from "./Grant.js";
import { type Heading } from "./Heading.js";
import { type IfBlock } from "./IfBlock.js";
import { type IfBlockClause } from "./IfBlockClause.js";
import { type ImageObject } from "./ImageObject.js";
import { type IncludeBlock } from "./IncludeBlock.js";
import { type InsertBlock } from "./InsertBlock.js";
import { type InsertInline } from "./InsertInline.js";
import { type InstructionBlock } from "./InstructionBlock.js";
import { type InstructionInline } from "./InstructionInline.js";
import { type InstructionMessage } from "./InstructionMessage.js";
import { type Integer } from "./Integer.js";
import { type IntegerValidator } from "./IntegerValidator.js";
import { type Link } from "./Link.js";
import { type List } from "./List.js";
import { type ListItem } from "./ListItem.js";
import { type MathBlock } from "./MathBlock.js";
import { type MathInline } from "./MathInline.js";
import { type MediaObject } from "./MediaObject.js";
import { type ModelParameters } from "./ModelParameters.js";
import { type ModifyBlock } from "./ModifyBlock.js";
import { type ModifyInline } from "./ModifyInline.js";
import { type ModifyOperation } from "./ModifyOperation.js";
import { type MonetaryGrant } from "./MonetaryGrant.js";
import { type Note } from "./Note.js";
import { type NumberValidator } from "./NumberValidator.js";
import { type Object } from "./Object.js";
import { type ObjectHint } from "./ObjectHint.js";
import { type Organization } from "./Organization.js";
import { type Paragraph } from "./Paragraph.js";
import { type Parameter } from "./Parameter.js";
import { type Periodical } from "./Periodical.js";
import { type Person } from "./Person.js";
import { type PostalAddress } from "./PostalAddress.js";
import { type Product } from "./Product.js";
import { type Prompt } from "./Prompt.js";
import { type PromptBlock } from "./PromptBlock.js";
import { type PropertyValue } from "./PropertyValue.js";
import { type ProvenanceCount } from "./ProvenanceCount.js";
import { type PublicationIssue } from "./PublicationIssue.js";
import { type PublicationVolume } from "./PublicationVolume.js";
import { type QuoteBlock } from "./QuoteBlock.js";
import { type QuoteInline } from "./QuoteInline.js";
import { type RawBlock } from "./RawBlock.js";
import { type ReplaceBlock } from "./ReplaceBlock.js";
import { type ReplaceInline } from "./ReplaceInline.js";
import { type Review } from "./Review.js";
import { type Section } from "./Section.js";
import { type SoftwareApplication } from "./SoftwareApplication.js";
import { type SoftwareSourceCode } from "./SoftwareSourceCode.js";
import { type Strikeout } from "./Strikeout.js";
import { type StringHint } from "./StringHint.js";
import { type StringOperation } from "./StringOperation.js";
import { type StringPatch } from "./StringPatch.js";
import { type StringValidator } from "./StringValidator.js";
import { type Strong } from "./Strong.js";
import { type StyledBlock } from "./StyledBlock.js";
import { type StyledInline } from "./StyledInline.js";
import { type Subscript } from "./Subscript.js";
import { type SuggestionBlock } from "./SuggestionBlock.js";
import { type SuggestionInline } from "./SuggestionInline.js";
import { type Superscript } from "./Superscript.js";
import { type Table } from "./Table.js";
import { type TableCell } from "./TableCell.js";
import { type TableRow } from "./TableRow.js";
import { type Text } from "./Text.js";
import { type ThematicBreak } from "./ThematicBreak.js";
import { type Thing } from "./Thing.js";
import { type Time } from "./Time.js";
import { type TimeValidator } from "./TimeValidator.js";
import { type Timestamp } from "./Timestamp.js";
import { type TimestampValidator } from "./TimestampValidator.js";
import { type TupleValidator } from "./TupleValidator.js";
import { type Underline } from "./Underline.js";
import { type Unknown } from "./Unknown.js";
import { type UnsignedInteger } from "./UnsignedInteger.js";
import { type Variable } from "./Variable.js";
import { type VideoObject } from "./VideoObject.js";
import { type Walkthrough } from "./Walkthrough.js";
import { type WalkthroughStep } from "./WalkthroughStep.js";

/**
 * Union type for all types in this schema, including primitives and entities
 */
export type Node =
  null |
  boolean |
  Integer |
  UnsignedInteger |
  number |
  string |
  Cord |
  Array |
  Admonition |
  ArrayHint |
  ArrayValidator |
  Article |
  AudioObject |
  AuthorRole |
  BooleanValidator |
  Brand |
  Button |
  CallArgument |
  CallBlock |
  Chat |
  ChatMessage |
  ChatMessageGroup |
  Cite |
  CiteGroup |
  Claim |
  CodeBlock |
  CodeChunk |
  CodeExpression |
  CodeInline |
  CodeLocation |
  Collection |
  Comment |
  CompilationDigest |
  CompilationMessage |
  Config |
  ConstantValidator |
  ContactPoint |
  CreativeWork |
  Datatable |
  DatatableColumn |
  DatatableColumnHint |
  DatatableHint |
  Date |
  DateTime |
  DateTimeValidator |
  DateValidator |
  DefinedTerm |
  DeleteBlock |
  DeleteInline |
  Directory |
  Duration |
  DurationValidator |
  Emphasis |
  EnumValidator |
  Enumeration |
  ExecutionDependant |
  ExecutionDependency |
  ExecutionMessage |
  ExecutionTag |
  Figure |
  File |
  ForBlock |
  Form |
  Function |
  Grant |
  Heading |
  IfBlock |
  IfBlockClause |
  ImageObject |
  IncludeBlock |
  InsertBlock |
  InsertInline |
  InstructionBlock |
  InstructionInline |
  InstructionMessage |
  IntegerValidator |
  Link |
  List |
  ListItem |
  MathBlock |
  MathInline |
  MediaObject |
  ModelParameters |
  ModifyBlock |
  ModifyInline |
  ModifyOperation |
  MonetaryGrant |
  Note |
  NumberValidator |
  ObjectHint |
  Organization |
  Paragraph |
  Parameter |
  Periodical |
  Person |
  PostalAddress |
  Product |
  Prompt |
  PromptBlock |
  PropertyValue |
  ProvenanceCount |
  PublicationIssue |
  PublicationVolume |
  QuoteBlock |
  QuoteInline |
  RawBlock |
  ReplaceBlock |
  ReplaceInline |
  Review |
  Section |
  SoftwareApplication |
  SoftwareSourceCode |
  Strikeout |
  StringHint |
  StringOperation |
  StringPatch |
  StringValidator |
  Strong |
  StyledBlock |
  StyledInline |
  Subscript |
  SuggestionBlock |
  SuggestionInline |
  Superscript |
  Table |
  TableCell |
  TableRow |
  Text |
  ThematicBreak |
  Thing |
  Time |
  TimeValidator |
  Timestamp |
  TimestampValidator |
  TupleValidator |
  Underline |
  Unknown |
  Variable |
  VideoObject |
  Walkthrough |
  WalkthroughStep |
  Object;

/**
 * Create a `Node` from an object
 */
export function node(other: Node): Node {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as Node;
  }
  switch(other.type) {
    case "Admonition":
    case "ArrayHint":
    case "ArrayValidator":
    case "Article":
    case "AudioObject":
    case "AuthorRole":
    case "BooleanValidator":
    case "Brand":
    case "Button":
    case "CallArgument":
    case "CallBlock":
    case "Chat":
    case "ChatMessage":
    case "ChatMessageGroup":
    case "Cite":
    case "CiteGroup":
    case "Claim":
    case "CodeBlock":
    case "CodeChunk":
    case "CodeExpression":
    case "CodeInline":
    case "CodeLocation":
    case "Collection":
    case "Comment":
    case "CompilationDigest":
    case "CompilationMessage":
    case "Config":
    case "ConstantValidator":
    case "ContactPoint":
    case "CreativeWork":
    case "Datatable":
    case "DatatableColumn":
    case "DatatableColumnHint":
    case "DatatableHint":
    case "Date":
    case "DateTime":
    case "DateTimeValidator":
    case "DateValidator":
    case "DefinedTerm":
    case "DeleteBlock":
    case "DeleteInline":
    case "Directory":
    case "Duration":
    case "DurationValidator":
    case "Emphasis":
    case "EnumValidator":
    case "Enumeration":
    case "ExecutionDependant":
    case "ExecutionDependency":
    case "ExecutionMessage":
    case "ExecutionTag":
    case "Figure":
    case "File":
    case "ForBlock":
    case "Form":
    case "Function":
    case "Grant":
    case "Heading":
    case "IfBlock":
    case "IfBlockClause":
    case "ImageObject":
    case "IncludeBlock":
    case "InsertBlock":
    case "InsertInline":
    case "InstructionBlock":
    case "InstructionInline":
    case "InstructionMessage":
    case "IntegerValidator":
    case "Link":
    case "List":
    case "ListItem":
    case "MathBlock":
    case "MathInline":
    case "MediaObject":
    case "ModelParameters":
    case "ModifyBlock":
    case "ModifyInline":
    case "ModifyOperation":
    case "MonetaryGrant":
    case "Note":
    case "NumberValidator":
    case "ObjectHint":
    case "Organization":
    case "Paragraph":
    case "Parameter":
    case "Periodical":
    case "Person":
    case "PostalAddress":
    case "Product":
    case "Prompt":
    case "PromptBlock":
    case "PropertyValue":
    case "ProvenanceCount":
    case "PublicationIssue":
    case "PublicationVolume":
    case "QuoteBlock":
    case "QuoteInline":
    case "RawBlock":
    case "ReplaceBlock":
    case "ReplaceInline":
    case "Review":
    case "Section":
    case "SoftwareApplication":
    case "SoftwareSourceCode":
    case "Strikeout":
    case "StringHint":
    case "StringOperation":
    case "StringPatch":
    case "StringValidator":
    case "Strong":
    case "StyledBlock":
    case "StyledInline":
    case "Subscript":
    case "SuggestionBlock":
    case "SuggestionInline":
    case "Superscript":
    case "Table":
    case "TableCell":
    case "TableRow":
    case "Text":
    case "ThematicBreak":
    case "Thing":
    case "Time":
    case "TimeValidator":
    case "Timestamp":
    case "TimestampValidator":
    case "TupleValidator":
    case "Underline":
    case "Unknown":
    case "Variable":
    case "VideoObject":
    case "Walkthrough":
    case "WalkthroughStep":
      return hydrate(other) as Node
    default:
      throw new Error(`Unexpected type for Node: ${other.type}`);
  }
}
