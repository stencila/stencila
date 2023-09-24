// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Array } from './Array'
import { ArrayValidator } from './ArrayValidator'
import { Article } from './Article'
import { AudioObject } from './AudioObject'
import { BooleanValidator } from './BooleanValidator'
import { Brand } from './Brand'
import { Button } from './Button'
import { Call } from './Call'
import { CallArgument } from './CallArgument'
import { Cite } from './Cite'
import { CiteGroup } from './CiteGroup'
import { Claim } from './Claim'
import { CodeBlock } from './CodeBlock'
import { CodeChunk } from './CodeChunk'
import { CodeError } from './CodeError'
import { CodeExpression } from './CodeExpression'
import { CodeFragment } from './CodeFragment'
import { Collection } from './Collection'
import { Comment } from './Comment'
import { ConstantValidator } from './ConstantValidator'
import { ContactPoint } from './ContactPoint'
import { Cord } from './Cord'
import { CreativeWork } from './CreativeWork'
import { Datatable } from './Datatable'
import { DatatableColumn } from './DatatableColumn'
import { Date } from './Date'
import { DateTime } from './DateTime'
import { DateTimeValidator } from './DateTimeValidator'
import { DateValidator } from './DateValidator'
import { DefinedTerm } from './DefinedTerm'
import { Delete } from './Delete'
import { Directory } from './Directory'
import { Division } from './Division'
import { Duration } from './Duration'
import { DurationValidator } from './DurationValidator'
import { Emphasis } from './Emphasis'
import { EnumValidator } from './EnumValidator'
import { Enumeration } from './Enumeration'
import { ExecutionDependant } from './ExecutionDependant'
import { ExecutionDependency } from './ExecutionDependency'
import { ExecutionDigest } from './ExecutionDigest'
import { ExecutionTag } from './ExecutionTag'
import { Figure } from './Figure'
import { File } from './File'
import { For } from './For'
import { Form } from './Form'
import { Function } from './Function'
import { Grant } from './Grant'
import { Heading } from './Heading'
import { If } from './If'
import { IfClause } from './IfClause'
import { ImageObject } from './ImageObject'
import { Include } from './Include'
import { Insert } from './Insert'
import { Integer } from './Integer'
import { IntegerValidator } from './IntegerValidator'
import { Link } from './Link'
import { List } from './List'
import { ListItem } from './ListItem'
import { MathBlock } from './MathBlock'
import { MathFragment } from './MathFragment'
import { MediaObject } from './MediaObject'
import { MonetaryGrant } from './MonetaryGrant'
import { Note } from './Note'
import { NumberValidator } from './NumberValidator'
import { Object } from './Object'
import { Organization } from './Organization'
import { Paragraph } from './Paragraph'
import { Parameter } from './Parameter'
import { Periodical } from './Periodical'
import { Person } from './Person'
import { PostalAddress } from './PostalAddress'
import { Product } from './Product'
import { PropertyValue } from './PropertyValue'
import { PublicationIssue } from './PublicationIssue'
import { PublicationVolume } from './PublicationVolume'
import { Quote } from './Quote'
import { QuoteBlock } from './QuoteBlock'
import { Review } from './Review'
import { SoftwareApplication } from './SoftwareApplication'
import { SoftwareSourceCode } from './SoftwareSourceCode'
import { Span } from './Span'
import { Strikeout } from './Strikeout'
import { StringValidator } from './StringValidator'
import { Strong } from './Strong'
import { Subscript } from './Subscript'
import { Superscript } from './Superscript'
import { Table } from './Table'
import { TableCell } from './TableCell'
import { TableRow } from './TableRow'
import { Text } from './Text'
import { ThematicBreak } from './ThematicBreak'
import { Thing } from './Thing'
import { Time } from './Time'
import { TimeValidator } from './TimeValidator'
import { Timestamp } from './Timestamp'
import { TimestampValidator } from './TimestampValidator'
import { TupleValidator } from './TupleValidator'
import { Underline } from './Underline'
import { UnsignedInteger } from './UnsignedInteger'
import { Variable } from './Variable'
import { VideoObject } from './VideoObject'

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

export function node(other: Node): Node {
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
    default: throw new Error(`Unexpected type for Node: ${other.type}`)
  }
}
