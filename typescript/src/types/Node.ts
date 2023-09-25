// Generated file; do not edit. See `../rust/schema-gen` crate.
            
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

// Union type for all types in this schema, including primitives and entities
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

export function nodeFrom(other: Node): Node {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as Node;
  }
  switch(other.type) {
    case "ArrayValidator": return ArrayValidator.from(other as ArrayValidator);
    case "Article": return Article.from(other as Article);
    case "AudioObject": return AudioObject.from(other as AudioObject);
    case "BooleanValidator": return BooleanValidator.from(other as BooleanValidator);
    case "Brand": return Brand.from(other as Brand);
    case "Button": return Button.from(other as Button);
    case "Call": return Call.from(other as Call);
    case "CallArgument": return CallArgument.from(other as CallArgument);
    case "Cite": return Cite.from(other as Cite);
    case "CiteGroup": return CiteGroup.from(other as CiteGroup);
    case "Claim": return Claim.from(other as Claim);
    case "CodeBlock": return CodeBlock.from(other as CodeBlock);
    case "CodeChunk": return CodeChunk.from(other as CodeChunk);
    case "CodeError": return CodeError.from(other as CodeError);
    case "CodeExpression": return CodeExpression.from(other as CodeExpression);
    case "CodeFragment": return CodeFragment.from(other as CodeFragment);
    case "Collection": return Collection.from(other as Collection);
    case "Comment": return Comment.from(other as Comment);
    case "ConstantValidator": return ConstantValidator.from(other as ConstantValidator);
    case "ContactPoint": return ContactPoint.from(other as ContactPoint);
    case "CreativeWork": return CreativeWork.from(other as CreativeWork);
    case "Datatable": return Datatable.from(other as Datatable);
    case "DatatableColumn": return DatatableColumn.from(other as DatatableColumn);
    case "Date": return Date.from(other as Date);
    case "DateTime": return DateTime.from(other as DateTime);
    case "DateTimeValidator": return DateTimeValidator.from(other as DateTimeValidator);
    case "DateValidator": return DateValidator.from(other as DateValidator);
    case "DefinedTerm": return DefinedTerm.from(other as DefinedTerm);
    case "Delete": return Delete.from(other as Delete);
    case "Directory": return Directory.from(other as Directory);
    case "Division": return Division.from(other as Division);
    case "Duration": return Duration.from(other as Duration);
    case "DurationValidator": return DurationValidator.from(other as DurationValidator);
    case "Emphasis": return Emphasis.from(other as Emphasis);
    case "EnumValidator": return EnumValidator.from(other as EnumValidator);
    case "Enumeration": return Enumeration.from(other as Enumeration);
    case "ExecutionDependant": return ExecutionDependant.from(other as ExecutionDependant);
    case "ExecutionDependency": return ExecutionDependency.from(other as ExecutionDependency);
    case "ExecutionDigest": return ExecutionDigest.from(other as ExecutionDigest);
    case "ExecutionTag": return ExecutionTag.from(other as ExecutionTag);
    case "Figure": return Figure.from(other as Figure);
    case "File": return File.from(other as File);
    case "For": return For.from(other as For);
    case "Form": return Form.from(other as Form);
    case "Function": return Function.from(other as Function);
    case "Grant": return Grant.from(other as Grant);
    case "Heading": return Heading.from(other as Heading);
    case "If": return If.from(other as If);
    case "IfClause": return IfClause.from(other as IfClause);
    case "ImageObject": return ImageObject.from(other as ImageObject);
    case "Include": return Include.from(other as Include);
    case "Insert": return Insert.from(other as Insert);
    case "IntegerValidator": return IntegerValidator.from(other as IntegerValidator);
    case "Link": return Link.from(other as Link);
    case "List": return List.from(other as List);
    case "ListItem": return ListItem.from(other as ListItem);
    case "MathBlock": return MathBlock.from(other as MathBlock);
    case "MathFragment": return MathFragment.from(other as MathFragment);
    case "MediaObject": return MediaObject.from(other as MediaObject);
    case "MonetaryGrant": return MonetaryGrant.from(other as MonetaryGrant);
    case "Note": return Note.from(other as Note);
    case "NumberValidator": return NumberValidator.from(other as NumberValidator);
    case "Organization": return Organization.from(other as Organization);
    case "Paragraph": return Paragraph.from(other as Paragraph);
    case "Parameter": return Parameter.from(other as Parameter);
    case "Periodical": return Periodical.from(other as Periodical);
    case "Person": return Person.from(other as Person);
    case "PostalAddress": return PostalAddress.from(other as PostalAddress);
    case "Product": return Product.from(other as Product);
    case "PropertyValue": return PropertyValue.from(other as PropertyValue);
    case "PublicationIssue": return PublicationIssue.from(other as PublicationIssue);
    case "PublicationVolume": return PublicationVolume.from(other as PublicationVolume);
    case "Quote": return Quote.from(other as Quote);
    case "QuoteBlock": return QuoteBlock.from(other as QuoteBlock);
    case "Review": return Review.from(other as Review);
    case "SoftwareApplication": return SoftwareApplication.from(other as SoftwareApplication);
    case "SoftwareSourceCode": return SoftwareSourceCode.from(other as SoftwareSourceCode);
    case "Span": return Span.from(other as Span);
    case "Strikeout": return Strikeout.from(other as Strikeout);
    case "StringValidator": return StringValidator.from(other as StringValidator);
    case "Strong": return Strong.from(other as Strong);
    case "Subscript": return Subscript.from(other as Subscript);
    case "Superscript": return Superscript.from(other as Superscript);
    case "Table": return Table.from(other as Table);
    case "TableCell": return TableCell.from(other as TableCell);
    case "TableRow": return TableRow.from(other as TableRow);
    case "Text": return Text.from(other as Text);
    case "ThematicBreak": return ThematicBreak.from(other as ThematicBreak);
    case "Thing": return Thing.from(other as Thing);
    case "Time": return Time.from(other as Time);
    case "TimeValidator": return TimeValidator.from(other as TimeValidator);
    case "Timestamp": return Timestamp.from(other as Timestamp);
    case "TimestampValidator": return TimestampValidator.from(other as TimestampValidator);
    case "TupleValidator": return TupleValidator.from(other as TupleValidator);
    case "Underline": return Underline.from(other as Underline);
    case "Variable": return Variable.from(other as Variable);
    case "VideoObject": return VideoObject.from(other as VideoObject);
    default: throw new Error(`Unexpected type for Node: ${other.type}`);
  }
}
