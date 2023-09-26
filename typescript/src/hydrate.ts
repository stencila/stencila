import * as types from "./types/index.js";

/**
 * Hydrate a value to a class instance if appropriate
 * 
 * @param value The value to hydrate into a class
 * @returns The value, hydrated into a class instance if appropriate
 */
export function hydrate(value: types.Node): types.Node {
  if (value == null || typeof value !== "object") {
    return value as types.Node;
  }

  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index++) {
      // @ts-expect-error because hydrate returns any node type
      value[index] = hydrate(value[index] as types.Node);
    }
    return value;
  }

  if (typeof value.type === "undefined") {
    return value;
  }

  for (const prop in value) {
    // @ts-expect-error because hydrate returns any node type
    value[prop] = hydrate(value[prop]);
  }

  switch (value.type) {
    // Generated code, do not exit
    // TYPE-CASES:START
    case "ArrayValidator":
      return value instanceof types.ArrayValidator ? value : Object.setPrototypeOf(value, types.ArrayValidator.prototype);
    case "Article":
      return value instanceof types.Article ? value : Object.setPrototypeOf(value, types.Article.prototype);
    case "AudioObject":
      return value instanceof types.AudioObject ? value : Object.setPrototypeOf(value, types.AudioObject.prototype);
    case "BooleanValidator":
      return value instanceof types.BooleanValidator ? value : Object.setPrototypeOf(value, types.BooleanValidator.prototype);
    case "Brand":
      return value instanceof types.Brand ? value : Object.setPrototypeOf(value, types.Brand.prototype);
    case "Button":
      return value instanceof types.Button ? value : Object.setPrototypeOf(value, types.Button.prototype);
    case "Call":
      return value instanceof types.Call ? value : Object.setPrototypeOf(value, types.Call.prototype);
    case "CallArgument":
      return value instanceof types.CallArgument ? value : Object.setPrototypeOf(value, types.CallArgument.prototype);
    case "Cite":
      return value instanceof types.Cite ? value : Object.setPrototypeOf(value, types.Cite.prototype);
    case "CiteGroup":
      return value instanceof types.CiteGroup ? value : Object.setPrototypeOf(value, types.CiteGroup.prototype);
    case "Claim":
      return value instanceof types.Claim ? value : Object.setPrototypeOf(value, types.Claim.prototype);
    case "CodeBlock":
      return value instanceof types.CodeBlock ? value : Object.setPrototypeOf(value, types.CodeBlock.prototype);
    case "CodeChunk":
      return value instanceof types.CodeChunk ? value : Object.setPrototypeOf(value, types.CodeChunk.prototype);
    case "CodeError":
      return value instanceof types.CodeError ? value : Object.setPrototypeOf(value, types.CodeError.prototype);
    case "CodeExecutable":
      return value instanceof types.CodeExecutable ? value : Object.setPrototypeOf(value, types.CodeExecutable.prototype);
    case "CodeExpression":
      return value instanceof types.CodeExpression ? value : Object.setPrototypeOf(value, types.CodeExpression.prototype);
    case "CodeFragment":
      return value instanceof types.CodeFragment ? value : Object.setPrototypeOf(value, types.CodeFragment.prototype);
    case "CodeStatic":
      return value instanceof types.CodeStatic ? value : Object.setPrototypeOf(value, types.CodeStatic.prototype);
    case "Collection":
      return value instanceof types.Collection ? value : Object.setPrototypeOf(value, types.Collection.prototype);
    case "Comment":
      return value instanceof types.Comment ? value : Object.setPrototypeOf(value, types.Comment.prototype);
    case "ConstantValidator":
      return value instanceof types.ConstantValidator ? value : Object.setPrototypeOf(value, types.ConstantValidator.prototype);
    case "ContactPoint":
      return value instanceof types.ContactPoint ? value : Object.setPrototypeOf(value, types.ContactPoint.prototype);
    case "CreativeWork":
      return value instanceof types.CreativeWork ? value : Object.setPrototypeOf(value, types.CreativeWork.prototype);
    case "Datatable":
      return value instanceof types.Datatable ? value : Object.setPrototypeOf(value, types.Datatable.prototype);
    case "DatatableColumn":
      return value instanceof types.DatatableColumn ? value : Object.setPrototypeOf(value, types.DatatableColumn.prototype);
    case "Date":
      return value instanceof types.Date ? value : Object.setPrototypeOf(value, types.Date.prototype);
    case "DateTime":
      return value instanceof types.DateTime ? value : Object.setPrototypeOf(value, types.DateTime.prototype);
    case "DateTimeValidator":
      return value instanceof types.DateTimeValidator ? value : Object.setPrototypeOf(value, types.DateTimeValidator.prototype);
    case "DateValidator":
      return value instanceof types.DateValidator ? value : Object.setPrototypeOf(value, types.DateValidator.prototype);
    case "DefinedTerm":
      return value instanceof types.DefinedTerm ? value : Object.setPrototypeOf(value, types.DefinedTerm.prototype);
    case "Delete":
      return value instanceof types.Delete ? value : Object.setPrototypeOf(value, types.Delete.prototype);
    case "Directory":
      return value instanceof types.Directory ? value : Object.setPrototypeOf(value, types.Directory.prototype);
    case "Division":
      return value instanceof types.Division ? value : Object.setPrototypeOf(value, types.Division.prototype);
    case "Duration":
      return value instanceof types.Duration ? value : Object.setPrototypeOf(value, types.Duration.prototype);
    case "DurationValidator":
      return value instanceof types.DurationValidator ? value : Object.setPrototypeOf(value, types.DurationValidator.prototype);
    case "Emphasis":
      return value instanceof types.Emphasis ? value : Object.setPrototypeOf(value, types.Emphasis.prototype);
    case "Entity":
      return value instanceof types.Entity ? value : Object.setPrototypeOf(value, types.Entity.prototype);
    case "EnumValidator":
      return value instanceof types.EnumValidator ? value : Object.setPrototypeOf(value, types.EnumValidator.prototype);
    case "Enumeration":
      return value instanceof types.Enumeration ? value : Object.setPrototypeOf(value, types.Enumeration.prototype);
    case "Executable":
      return value instanceof types.Executable ? value : Object.setPrototypeOf(value, types.Executable.prototype);
    case "ExecutionDependant":
      return value instanceof types.ExecutionDependant ? value : Object.setPrototypeOf(value, types.ExecutionDependant.prototype);
    case "ExecutionDependency":
      return value instanceof types.ExecutionDependency ? value : Object.setPrototypeOf(value, types.ExecutionDependency.prototype);
    case "ExecutionDigest":
      return value instanceof types.ExecutionDigest ? value : Object.setPrototypeOf(value, types.ExecutionDigest.prototype);
    case "ExecutionTag":
      return value instanceof types.ExecutionTag ? value : Object.setPrototypeOf(value, types.ExecutionTag.prototype);
    case "Figure":
      return value instanceof types.Figure ? value : Object.setPrototypeOf(value, types.Figure.prototype);
    case "File":
      return value instanceof types.File ? value : Object.setPrototypeOf(value, types.File.prototype);
    case "For":
      return value instanceof types.For ? value : Object.setPrototypeOf(value, types.For.prototype);
    case "Form":
      return value instanceof types.Form ? value : Object.setPrototypeOf(value, types.Form.prototype);
    case "Function":
      return value instanceof types.Function ? value : Object.setPrototypeOf(value, types.Function.prototype);
    case "Grant":
      return value instanceof types.Grant ? value : Object.setPrototypeOf(value, types.Grant.prototype);
    case "Heading":
      return value instanceof types.Heading ? value : Object.setPrototypeOf(value, types.Heading.prototype);
    case "If":
      return value instanceof types.If ? value : Object.setPrototypeOf(value, types.If.prototype);
    case "IfClause":
      return value instanceof types.IfClause ? value : Object.setPrototypeOf(value, types.IfClause.prototype);
    case "ImageObject":
      return value instanceof types.ImageObject ? value : Object.setPrototypeOf(value, types.ImageObject.prototype);
    case "Include":
      return value instanceof types.Include ? value : Object.setPrototypeOf(value, types.Include.prototype);
    case "Insert":
      return value instanceof types.Insert ? value : Object.setPrototypeOf(value, types.Insert.prototype);
    case "IntegerValidator":
      return value instanceof types.IntegerValidator ? value : Object.setPrototypeOf(value, types.IntegerValidator.prototype);
    case "Link":
      return value instanceof types.Link ? value : Object.setPrototypeOf(value, types.Link.prototype);
    case "List":
      return value instanceof types.List ? value : Object.setPrototypeOf(value, types.List.prototype);
    case "ListItem":
      return value instanceof types.ListItem ? value : Object.setPrototypeOf(value, types.ListItem.prototype);
    case "Mark":
      return value instanceof types.Mark ? value : Object.setPrototypeOf(value, types.Mark.prototype);
    case "Math":
      return value instanceof types.Math ? value : Object.setPrototypeOf(value, types.Math.prototype);
    case "MathBlock":
      return value instanceof types.MathBlock ? value : Object.setPrototypeOf(value, types.MathBlock.prototype);
    case "MathFragment":
      return value instanceof types.MathFragment ? value : Object.setPrototypeOf(value, types.MathFragment.prototype);
    case "MediaObject":
      return value instanceof types.MediaObject ? value : Object.setPrototypeOf(value, types.MediaObject.prototype);
    case "MonetaryGrant":
      return value instanceof types.MonetaryGrant ? value : Object.setPrototypeOf(value, types.MonetaryGrant.prototype);
    case "Note":
      return value instanceof types.Note ? value : Object.setPrototypeOf(value, types.Note.prototype);
    case "NumberValidator":
      return value instanceof types.NumberValidator ? value : Object.setPrototypeOf(value, types.NumberValidator.prototype);
    case "Organization":
      return value instanceof types.Organization ? value : Object.setPrototypeOf(value, types.Organization.prototype);
    case "Paragraph":
      return value instanceof types.Paragraph ? value : Object.setPrototypeOf(value, types.Paragraph.prototype);
    case "Parameter":
      return value instanceof types.Parameter ? value : Object.setPrototypeOf(value, types.Parameter.prototype);
    case "Periodical":
      return value instanceof types.Periodical ? value : Object.setPrototypeOf(value, types.Periodical.prototype);
    case "Person":
      return value instanceof types.Person ? value : Object.setPrototypeOf(value, types.Person.prototype);
    case "PostalAddress":
      return value instanceof types.PostalAddress ? value : Object.setPrototypeOf(value, types.PostalAddress.prototype);
    case "Product":
      return value instanceof types.Product ? value : Object.setPrototypeOf(value, types.Product.prototype);
    case "PropertyValue":
      return value instanceof types.PropertyValue ? value : Object.setPrototypeOf(value, types.PropertyValue.prototype);
    case "PublicationIssue":
      return value instanceof types.PublicationIssue ? value : Object.setPrototypeOf(value, types.PublicationIssue.prototype);
    case "PublicationVolume":
      return value instanceof types.PublicationVolume ? value : Object.setPrototypeOf(value, types.PublicationVolume.prototype);
    case "Quote":
      return value instanceof types.Quote ? value : Object.setPrototypeOf(value, types.Quote.prototype);
    case "QuoteBlock":
      return value instanceof types.QuoteBlock ? value : Object.setPrototypeOf(value, types.QuoteBlock.prototype);
    case "Review":
      return value instanceof types.Review ? value : Object.setPrototypeOf(value, types.Review.prototype);
    case "SoftwareApplication":
      return value instanceof types.SoftwareApplication ? value : Object.setPrototypeOf(value, types.SoftwareApplication.prototype);
    case "SoftwareSourceCode":
      return value instanceof types.SoftwareSourceCode ? value : Object.setPrototypeOf(value, types.SoftwareSourceCode.prototype);
    case "Span":
      return value instanceof types.Span ? value : Object.setPrototypeOf(value, types.Span.prototype);
    case "Strikeout":
      return value instanceof types.Strikeout ? value : Object.setPrototypeOf(value, types.Strikeout.prototype);
    case "StringValidator":
      return value instanceof types.StringValidator ? value : Object.setPrototypeOf(value, types.StringValidator.prototype);
    case "Strong":
      return value instanceof types.Strong ? value : Object.setPrototypeOf(value, types.Strong.prototype);
    case "Styled":
      return value instanceof types.Styled ? value : Object.setPrototypeOf(value, types.Styled.prototype);
    case "Subscript":
      return value instanceof types.Subscript ? value : Object.setPrototypeOf(value, types.Subscript.prototype);
    case "Suggestion":
      return value instanceof types.Suggestion ? value : Object.setPrototypeOf(value, types.Suggestion.prototype);
    case "Superscript":
      return value instanceof types.Superscript ? value : Object.setPrototypeOf(value, types.Superscript.prototype);
    case "Table":
      return value instanceof types.Table ? value : Object.setPrototypeOf(value, types.Table.prototype);
    case "TableCell":
      return value instanceof types.TableCell ? value : Object.setPrototypeOf(value, types.TableCell.prototype);
    case "TableRow":
      return value instanceof types.TableRow ? value : Object.setPrototypeOf(value, types.TableRow.prototype);
    case "Text":
      return value instanceof types.Text ? value : Object.setPrototypeOf(value, types.Text.prototype);
    case "ThematicBreak":
      return value instanceof types.ThematicBreak ? value : Object.setPrototypeOf(value, types.ThematicBreak.prototype);
    case "Thing":
      return value instanceof types.Thing ? value : Object.setPrototypeOf(value, types.Thing.prototype);
    case "Time":
      return value instanceof types.Time ? value : Object.setPrototypeOf(value, types.Time.prototype);
    case "TimeValidator":
      return value instanceof types.TimeValidator ? value : Object.setPrototypeOf(value, types.TimeValidator.prototype);
    case "Timestamp":
      return value instanceof types.Timestamp ? value : Object.setPrototypeOf(value, types.Timestamp.prototype);
    case "TimestampValidator":
      return value instanceof types.TimestampValidator ? value : Object.setPrototypeOf(value, types.TimestampValidator.prototype);
    case "TupleValidator":
      return value instanceof types.TupleValidator ? value : Object.setPrototypeOf(value, types.TupleValidator.prototype);
    case "Underline":
      return value instanceof types.Underline ? value : Object.setPrototypeOf(value, types.Underline.prototype);
    case "Variable":
      return value instanceof types.Variable ? value : Object.setPrototypeOf(value, types.Variable.prototype);
    case "VideoObject":
      return value instanceof types.VideoObject ? value : Object.setPrototypeOf(value, types.VideoObject.prototype);
    // TYPE-CASES:STOP
    default:
      return value;
  }
}
