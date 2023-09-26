// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { Array } from "./Array.js";
import { ArrayValidator } from "./ArrayValidator.js";
import { Article } from "./Article.js";
import { AudioObject } from "./AudioObject.js";
import { BooleanValidator } from "./BooleanValidator.js";
import { Brand } from "./Brand.js";
import { Button } from "./Button.js";
import { Call } from "./Call.js";
import { CallArgument } from "./CallArgument.js";
import { Cite } from "./Cite.js";
import { CiteGroup } from "./CiteGroup.js";
import { Claim } from "./Claim.js";
import { CodeBlock } from "./CodeBlock.js";
import { CodeChunk } from "./CodeChunk.js";
import { CodeError } from "./CodeError.js";
import { CodeExpression } from "./CodeExpression.js";
import { CodeFragment } from "./CodeFragment.js";
import { Collection } from "./Collection.js";
import { Comment } from "./Comment.js";
import { ConstantValidator } from "./ConstantValidator.js";
import { ContactPoint } from "./ContactPoint.js";
import { Cord } from "./Cord.js";
import { CreativeWork } from "./CreativeWork.js";
import { Datatable } from "./Datatable.js";
import { DatatableColumn } from "./DatatableColumn.js";
import { Date } from "./Date.js";
import { DateTime } from "./DateTime.js";
import { DateTimeValidator } from "./DateTimeValidator.js";
import { DateValidator } from "./DateValidator.js";
import { DefinedTerm } from "./DefinedTerm.js";
import { Delete } from "./Delete.js";
import { Directory } from "./Directory.js";
import { Division } from "./Division.js";
import { Duration } from "./Duration.js";
import { DurationValidator } from "./DurationValidator.js";
import { Emphasis } from "./Emphasis.js";
import { EnumValidator } from "./EnumValidator.js";
import { Enumeration } from "./Enumeration.js";
import { ExecutionDependant } from "./ExecutionDependant.js";
import { ExecutionDependency } from "./ExecutionDependency.js";
import { ExecutionDigest } from "./ExecutionDigest.js";
import { ExecutionTag } from "./ExecutionTag.js";
import { Figure } from "./Figure.js";
import { File } from "./File.js";
import { For } from "./For.js";
import { Form } from "./Form.js";
import { Function } from "./Function.js";
import { Grant } from "./Grant.js";
import { Heading } from "./Heading.js";
import { If } from "./If.js";
import { IfClause } from "./IfClause.js";
import { ImageObject } from "./ImageObject.js";
import { Include } from "./Include.js";
import { Insert } from "./Insert.js";
import { Integer } from "./Integer.js";
import { IntegerValidator } from "./IntegerValidator.js";
import { Link } from "./Link.js";
import { List } from "./List.js";
import { ListItem } from "./ListItem.js";
import { MathBlock } from "./MathBlock.js";
import { MathFragment } from "./MathFragment.js";
import { MediaObject } from "./MediaObject.js";
import { MonetaryGrant } from "./MonetaryGrant.js";
import { Note } from "./Note.js";
import { NumberValidator } from "./NumberValidator.js";
import { Object } from "./Object.js";
import { Organization } from "./Organization.js";
import { Paragraph } from "./Paragraph.js";
import { Parameter } from "./Parameter.js";
import { Periodical } from "./Periodical.js";
import { Person } from "./Person.js";
import { PostalAddress } from "./PostalAddress.js";
import { Product } from "./Product.js";
import { PropertyValue } from "./PropertyValue.js";
import { PublicationIssue } from "./PublicationIssue.js";
import { PublicationVolume } from "./PublicationVolume.js";
import { Quote } from "./Quote.js";
import { QuoteBlock } from "./QuoteBlock.js";
import { Review } from "./Review.js";
import { SoftwareApplication } from "./SoftwareApplication.js";
import { SoftwareSourceCode } from "./SoftwareSourceCode.js";
import { Span } from "./Span.js";
import { Strikeout } from "./Strikeout.js";
import { StringValidator } from "./StringValidator.js";
import { Strong } from "./Strong.js";
import { Subscript } from "./Subscript.js";
import { Superscript } from "./Superscript.js";
import { Table } from "./Table.js";
import { TableCell } from "./TableCell.js";
import { TableRow } from "./TableRow.js";
import { Text } from "./Text.js";
import { ThematicBreak } from "./ThematicBreak.js";
import { Thing } from "./Thing.js";
import { Time } from "./Time.js";
import { TimeValidator } from "./TimeValidator.js";
import { Timestamp } from "./Timestamp.js";
import { TimestampValidator } from "./TimestampValidator.js";
import { TupleValidator } from "./TupleValidator.js";
import { Underline } from "./Underline.js";
import { UnsignedInteger } from "./UnsignedInteger.js";
import { Variable } from "./Variable.js";
import { VideoObject } from "./VideoObject.js";

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
  CodeError |
  CodeExpression |
  CodeFragment |
  Collection |
  Comment |
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
  ExecutionDigest |
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
    case "CodeError":
    case "CodeExpression":
    case "CodeFragment":
    case "Collection":
    case "Comment":
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
    case "ExecutionDigest":
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
