// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Admonition } from "./Admonition.js";
import { type Array } from "./Array.js";
import { type ArrayValidator } from "./ArrayValidator.js";
import { type Article } from "./Article.js";
import { type AudioObject } from "./AudioObject.js";
import { type BooleanValidator } from "./BooleanValidator.js";
import { type Brand } from "./Brand.js";
import { type Button } from "./Button.js";
import { type Call } from "./Call.js";
import { type CallArgument } from "./CallArgument.js";
import { type Cite } from "./Cite.js";
import { type CiteGroup } from "./CiteGroup.js";
import { type Claim } from "./Claim.js";
import { type CodeBlock } from "./CodeBlock.js";
import { type CodeChunk } from "./CodeChunk.js";
import { type CodeExpression } from "./CodeExpression.js";
import { type CodeFragment } from "./CodeFragment.js";
import { type CodeLocation } from "./CodeLocation.js";
import { type Collection } from "./Collection.js";
import { type Comment } from "./Comment.js";
import { type CompilationDigest } from "./CompilationDigest.js";
import { type CompilationError } from "./CompilationError.js";
import { type ConstantValidator } from "./ConstantValidator.js";
import { type ContactPoint } from "./ContactPoint.js";
import { type Cord } from "./Cord.js";
import { type CreativeWork } from "./CreativeWork.js";
import { type Datatable } from "./Datatable.js";
import { type DatatableColumn } from "./DatatableColumn.js";
import { type Date } from "./Date.js";
import { type DateTime } from "./DateTime.js";
import { type DateTimeValidator } from "./DateTimeValidator.js";
import { type DateValidator } from "./DateValidator.js";
import { type DefinedTerm } from "./DefinedTerm.js";
import { type Delete } from "./Delete.js";
import { type Directory } from "./Directory.js";
import { type Division } from "./Division.js";
import { type Duration } from "./Duration.js";
import { type DurationValidator } from "./DurationValidator.js";
import { type Emphasis } from "./Emphasis.js";
import { type EnumValidator } from "./EnumValidator.js";
import { type Enumeration } from "./Enumeration.js";
import { type ExecutionDependant } from "./ExecutionDependant.js";
import { type ExecutionDependency } from "./ExecutionDependency.js";
import { type ExecutionError } from "./ExecutionError.js";
import { type ExecutionTag } from "./ExecutionTag.js";
import { type Figure } from "./Figure.js";
import { type File } from "./File.js";
import { type For } from "./For.js";
import { type Form } from "./Form.js";
import { type Function } from "./Function.js";
import { type Grant } from "./Grant.js";
import { type Heading } from "./Heading.js";
import { type If } from "./If.js";
import { type IfClause } from "./IfClause.js";
import { type ImageObject } from "./ImageObject.js";
import { type Include } from "./Include.js";
import { type Insert } from "./Insert.js";
import { type Integer } from "./Integer.js";
import { type IntegerValidator } from "./IntegerValidator.js";
import { type Link } from "./Link.js";
import { type List } from "./List.js";
import { type ListItem } from "./ListItem.js";
import { type MathBlock } from "./MathBlock.js";
import { type MathFragment } from "./MathFragment.js";
import { type MediaObject } from "./MediaObject.js";
import { type MonetaryGrant } from "./MonetaryGrant.js";
import { type Note } from "./Note.js";
import { type NumberValidator } from "./NumberValidator.js";
import { type Object } from "./Object.js";
import { type Organization } from "./Organization.js";
import { type Paragraph } from "./Paragraph.js";
import { type Parameter } from "./Parameter.js";
import { type Periodical } from "./Periodical.js";
import { type Person } from "./Person.js";
import { type PostalAddress } from "./PostalAddress.js";
import { type Product } from "./Product.js";
import { type PropertyValue } from "./PropertyValue.js";
import { type PublicationIssue } from "./PublicationIssue.js";
import { type PublicationVolume } from "./PublicationVolume.js";
import { type Quote } from "./Quote.js";
import { type QuoteBlock } from "./QuoteBlock.js";
import { type Review } from "./Review.js";
import { type Section } from "./Section.js";
import { type SoftwareApplication } from "./SoftwareApplication.js";
import { type SoftwareSourceCode } from "./SoftwareSourceCode.js";
import { type Span } from "./Span.js";
import { type Strikeout } from "./Strikeout.js";
import { type StringValidator } from "./StringValidator.js";
import { type Strong } from "./Strong.js";
import { type Subscript } from "./Subscript.js";
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
import { type UnsignedInteger } from "./UnsignedInteger.js";
import { type Variable } from "./Variable.js";
import { type VideoObject } from "./VideoObject.js";

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
  ArrayValidator |
  Article |
  AudioObject |
  BooleanValidator |
  Brand |
  Button |
  Call |
  CallArgument |
  Cite |
  CiteGroup |
  Claim |
  CodeBlock |
  CodeChunk |
  CodeExpression |
  CodeFragment |
  CodeLocation |
  Collection |
  Comment |
  CompilationDigest |
  CompilationError |
  ConstantValidator |
  ContactPoint |
  CreativeWork |
  Datatable |
  DatatableColumn |
  Date |
  DateTime |
  DateTimeValidator |
  DateValidator |
  DefinedTerm |
  Delete |
  Directory |
  Division |
  Duration |
  DurationValidator |
  Emphasis |
  EnumValidator |
  Enumeration |
  ExecutionDependant |
  ExecutionDependency |
  ExecutionError |
  ExecutionTag |
  Figure |
  File |
  For |
  Form |
  Function |
  Grant |
  Heading |
  If |
  IfClause |
  ImageObject |
  Include |
  Insert |
  IntegerValidator |
  Link |
  List |
  ListItem |
  MathBlock |
  MathFragment |
  MediaObject |
  MonetaryGrant |
  Note |
  NumberValidator |
  Organization |
  Paragraph |
  Parameter |
  Periodical |
  Person |
  PostalAddress |
  Product |
  PropertyValue |
  PublicationIssue |
  PublicationVolume |
  Quote |
  QuoteBlock |
  Review |
  Section |
  SoftwareApplication |
  SoftwareSourceCode |
  Span |
  Strikeout |
  StringValidator |
  Strong |
  Subscript |
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
  Variable |
  VideoObject |
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
    case "ArrayValidator":
    case "Article":
    case "AudioObject":
    case "BooleanValidator":
    case "Brand":
    case "Button":
    case "Call":
    case "CallArgument":
    case "Cite":
    case "CiteGroup":
    case "Claim":
    case "CodeBlock":
    case "CodeChunk":
    case "CodeExpression":
    case "CodeFragment":
    case "CodeLocation":
    case "Collection":
    case "Comment":
    case "CompilationDigest":
    case "CompilationError":
    case "ConstantValidator":
    case "ContactPoint":
    case "CreativeWork":
    case "Datatable":
    case "DatatableColumn":
    case "Date":
    case "DateTime":
    case "DateTimeValidator":
    case "DateValidator":
    case "DefinedTerm":
    case "Delete":
    case "Directory":
    case "Division":
    case "Duration":
    case "DurationValidator":
    case "Emphasis":
    case "EnumValidator":
    case "Enumeration":
    case "ExecutionDependant":
    case "ExecutionDependency":
    case "ExecutionError":
    case "ExecutionTag":
    case "Figure":
    case "File":
    case "For":
    case "Form":
    case "Function":
    case "Grant":
    case "Heading":
    case "If":
    case "IfClause":
    case "ImageObject":
    case "Include":
    case "Insert":
    case "IntegerValidator":
    case "Link":
    case "List":
    case "ListItem":
    case "MathBlock":
    case "MathFragment":
    case "MediaObject":
    case "MonetaryGrant":
    case "Note":
    case "NumberValidator":
    case "Organization":
    case "Paragraph":
    case "Parameter":
    case "Periodical":
    case "Person":
    case "PostalAddress":
    case "Product":
    case "PropertyValue":
    case "PublicationIssue":
    case "PublicationVolume":
    case "Quote":
    case "QuoteBlock":
    case "Review":
    case "Section":
    case "SoftwareApplication":
    case "SoftwareSourceCode":
    case "Span":
    case "Strikeout":
    case "StringValidator":
    case "Strong":
    case "Subscript":
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
    case "Variable":
    case "VideoObject":
      return hydrate(other) as Node
    default:
      throw new Error(`Unexpected type for Node: ${other.type}`);
  }
}
