import * as types from "./types/index.js";

/**
 * Hydrate a value to a class instance if appropriate
 *
 * @param value The value to hydrate into a class
 * @returns The value, hydrated into a class instance if appropriate
 */
export function hydrate(value: types.Node): types.Node {
  if (
    value == null ||
    typeof value !== "object" ||
    value instanceof types.Entity
  ) {
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
    case "Admonition":
      return Object.setPrototypeOf(value, types.Admonition.prototype);
    case "ArrayHint":
      return Object.setPrototypeOf(value, types.ArrayHint.prototype);
    case "ArrayValidator":
      return Object.setPrototypeOf(value, types.ArrayValidator.prototype);
    case "Article":
      return Object.setPrototypeOf(value, types.Article.prototype);
    case "AudioObject":
      return Object.setPrototypeOf(value, types.AudioObject.prototype);
    case "AuthorRole":
      return Object.setPrototypeOf(value, types.AuthorRole.prototype);
    case "BooleanValidator":
      return Object.setPrototypeOf(value, types.BooleanValidator.prototype);
    case "Brand":
      return Object.setPrototypeOf(value, types.Brand.prototype);
    case "Button":
      return Object.setPrototypeOf(value, types.Button.prototype);
    case "CallArgument":
      return Object.setPrototypeOf(value, types.CallArgument.prototype);
    case "CallBlock":
      return Object.setPrototypeOf(value, types.CallBlock.prototype);
    case "Chat":
      return Object.setPrototypeOf(value, types.Chat.prototype);
    case "ChatMessage":
      return Object.setPrototypeOf(value, types.ChatMessage.prototype);
    case "Cite":
      return Object.setPrototypeOf(value, types.Cite.prototype);
    case "CiteGroup":
      return Object.setPrototypeOf(value, types.CiteGroup.prototype);
    case "Claim":
      return Object.setPrototypeOf(value, types.Claim.prototype);
    case "CodeBlock":
      return Object.setPrototypeOf(value, types.CodeBlock.prototype);
    case "CodeChunk":
      return Object.setPrototypeOf(value, types.CodeChunk.prototype);
    case "CodeExecutable":
      return Object.setPrototypeOf(value, types.CodeExecutable.prototype);
    case "CodeExpression":
      return Object.setPrototypeOf(value, types.CodeExpression.prototype);
    case "CodeInline":
      return Object.setPrototypeOf(value, types.CodeInline.prototype);
    case "CodeLocation":
      return Object.setPrototypeOf(value, types.CodeLocation.prototype);
    case "CodeStatic":
      return Object.setPrototypeOf(value, types.CodeStatic.prototype);
    case "Collection":
      return Object.setPrototypeOf(value, types.Collection.prototype);
    case "Comment":
      return Object.setPrototypeOf(value, types.Comment.prototype);
    case "CompilationDigest":
      return Object.setPrototypeOf(value, types.CompilationDigest.prototype);
    case "CompilationMessage":
      return Object.setPrototypeOf(value, types.CompilationMessage.prototype);
    case "Config":
      return Object.setPrototypeOf(value, types.Config.prototype);
    case "ConstantValidator":
      return Object.setPrototypeOf(value, types.ConstantValidator.prototype);
    case "ContactPoint":
      return Object.setPrototypeOf(value, types.ContactPoint.prototype);
    case "CreativeWork":
      return Object.setPrototypeOf(value, types.CreativeWork.prototype);
    case "Datatable":
      return Object.setPrototypeOf(value, types.Datatable.prototype);
    case "DatatableColumn":
      return Object.setPrototypeOf(value, types.DatatableColumn.prototype);
    case "DatatableColumnHint":
      return Object.setPrototypeOf(value, types.DatatableColumnHint.prototype);
    case "DatatableHint":
      return Object.setPrototypeOf(value, types.DatatableHint.prototype);
    case "Date":
      return Object.setPrototypeOf(value, types.Date.prototype);
    case "DateTime":
      return Object.setPrototypeOf(value, types.DateTime.prototype);
    case "DateTimeValidator":
      return Object.setPrototypeOf(value, types.DateTimeValidator.prototype);
    case "DateValidator":
      return Object.setPrototypeOf(value, types.DateValidator.prototype);
    case "DefinedTerm":
      return Object.setPrototypeOf(value, types.DefinedTerm.prototype);
    case "DeleteBlock":
      return Object.setPrototypeOf(value, types.DeleteBlock.prototype);
    case "DeleteInline":
      return Object.setPrototypeOf(value, types.DeleteInline.prototype);
    case "Directory":
      return Object.setPrototypeOf(value, types.Directory.prototype);
    case "Duration":
      return Object.setPrototypeOf(value, types.Duration.prototype);
    case "DurationValidator":
      return Object.setPrototypeOf(value, types.DurationValidator.prototype);
    case "Emphasis":
      return Object.setPrototypeOf(value, types.Emphasis.prototype);
    case "Entity":
      return Object.setPrototypeOf(value, types.Entity.prototype);
    case "EnumValidator":
      return Object.setPrototypeOf(value, types.EnumValidator.prototype);
    case "Enumeration":
      return Object.setPrototypeOf(value, types.Enumeration.prototype);
    case "Executable":
      return Object.setPrototypeOf(value, types.Executable.prototype);
    case "ExecutionDependant":
      return Object.setPrototypeOf(value, types.ExecutionDependant.prototype);
    case "ExecutionDependency":
      return Object.setPrototypeOf(value, types.ExecutionDependency.prototype);
    case "ExecutionMessage":
      return Object.setPrototypeOf(value, types.ExecutionMessage.prototype);
    case "ExecutionTag":
      return Object.setPrototypeOf(value, types.ExecutionTag.prototype);
    case "Figure":
      return Object.setPrototypeOf(value, types.Figure.prototype);
    case "File":
      return Object.setPrototypeOf(value, types.File.prototype);
    case "ForBlock":
      return Object.setPrototypeOf(value, types.ForBlock.prototype);
    case "Form":
      return Object.setPrototypeOf(value, types.Form.prototype);
    case "Function":
      return Object.setPrototypeOf(value, types.Function.prototype);
    case "Grant":
      return Object.setPrototypeOf(value, types.Grant.prototype);
    case "Heading":
      return Object.setPrototypeOf(value, types.Heading.prototype);
    case "IfBlock":
      return Object.setPrototypeOf(value, types.IfBlock.prototype);
    case "IfBlockClause":
      return Object.setPrototypeOf(value, types.IfBlockClause.prototype);
    case "ImageObject":
      return Object.setPrototypeOf(value, types.ImageObject.prototype);
    case "IncludeBlock":
      return Object.setPrototypeOf(value, types.IncludeBlock.prototype);
    case "InsertBlock":
      return Object.setPrototypeOf(value, types.InsertBlock.prototype);
    case "InsertInline":
      return Object.setPrototypeOf(value, types.InsertInline.prototype);
    case "Instruction":
      return Object.setPrototypeOf(value, types.Instruction.prototype);
    case "InstructionBlock":
      return Object.setPrototypeOf(value, types.InstructionBlock.prototype);
    case "InstructionInline":
      return Object.setPrototypeOf(value, types.InstructionInline.prototype);
    case "InstructionMessage":
      return Object.setPrototypeOf(value, types.InstructionMessage.prototype);
    case "InstructionModel":
      return Object.setPrototypeOf(value, types.InstructionModel.prototype);
    case "IntegerValidator":
      return Object.setPrototypeOf(value, types.IntegerValidator.prototype);
    case "Link":
      return Object.setPrototypeOf(value, types.Link.prototype);
    case "List":
      return Object.setPrototypeOf(value, types.List.prototype);
    case "ListItem":
      return Object.setPrototypeOf(value, types.ListItem.prototype);
    case "Mark":
      return Object.setPrototypeOf(value, types.Mark.prototype);
    case "Math":
      return Object.setPrototypeOf(value, types.Math.prototype);
    case "MathBlock":
      return Object.setPrototypeOf(value, types.MathBlock.prototype);
    case "MathInline":
      return Object.setPrototypeOf(value, types.MathInline.prototype);
    case "MediaObject":
      return Object.setPrototypeOf(value, types.MediaObject.prototype);
    case "ModifyBlock":
      return Object.setPrototypeOf(value, types.ModifyBlock.prototype);
    case "ModifyInline":
      return Object.setPrototypeOf(value, types.ModifyInline.prototype);
    case "ModifyOperation":
      return Object.setPrototypeOf(value, types.ModifyOperation.prototype);
    case "MonetaryGrant":
      return Object.setPrototypeOf(value, types.MonetaryGrant.prototype);
    case "Note":
      return Object.setPrototypeOf(value, types.Note.prototype);
    case "NumberValidator":
      return Object.setPrototypeOf(value, types.NumberValidator.prototype);
    case "ObjectHint":
      return Object.setPrototypeOf(value, types.ObjectHint.prototype);
    case "Organization":
      return Object.setPrototypeOf(value, types.Organization.prototype);
    case "Paragraph":
      return Object.setPrototypeOf(value, types.Paragraph.prototype);
    case "Parameter":
      return Object.setPrototypeOf(value, types.Parameter.prototype);
    case "Periodical":
      return Object.setPrototypeOf(value, types.Periodical.prototype);
    case "Person":
      return Object.setPrototypeOf(value, types.Person.prototype);
    case "PostalAddress":
      return Object.setPrototypeOf(value, types.PostalAddress.prototype);
    case "Product":
      return Object.setPrototypeOf(value, types.Product.prototype);
    case "Prompt":
      return Object.setPrototypeOf(value, types.Prompt.prototype);
    case "PromptBlock":
      return Object.setPrototypeOf(value, types.PromptBlock.prototype);
    case "PropertyValue":
      return Object.setPrototypeOf(value, types.PropertyValue.prototype);
    case "ProvenanceCount":
      return Object.setPrototypeOf(value, types.ProvenanceCount.prototype);
    case "PublicationIssue":
      return Object.setPrototypeOf(value, types.PublicationIssue.prototype);
    case "PublicationVolume":
      return Object.setPrototypeOf(value, types.PublicationVolume.prototype);
    case "QuoteBlock":
      return Object.setPrototypeOf(value, types.QuoteBlock.prototype);
    case "QuoteInline":
      return Object.setPrototypeOf(value, types.QuoteInline.prototype);
    case "RawBlock":
      return Object.setPrototypeOf(value, types.RawBlock.prototype);
    case "ReplaceBlock":
      return Object.setPrototypeOf(value, types.ReplaceBlock.prototype);
    case "ReplaceInline":
      return Object.setPrototypeOf(value, types.ReplaceInline.prototype);
    case "Review":
      return Object.setPrototypeOf(value, types.Review.prototype);
    case "Role":
      return Object.setPrototypeOf(value, types.Role.prototype);
    case "Section":
      return Object.setPrototypeOf(value, types.Section.prototype);
    case "SoftwareApplication":
      return Object.setPrototypeOf(value, types.SoftwareApplication.prototype);
    case "SoftwareSourceCode":
      return Object.setPrototypeOf(value, types.SoftwareSourceCode.prototype);
    case "Strikeout":
      return Object.setPrototypeOf(value, types.Strikeout.prototype);
    case "StringHint":
      return Object.setPrototypeOf(value, types.StringHint.prototype);
    case "StringOperation":
      return Object.setPrototypeOf(value, types.StringOperation.prototype);
    case "StringPatch":
      return Object.setPrototypeOf(value, types.StringPatch.prototype);
    case "StringValidator":
      return Object.setPrototypeOf(value, types.StringValidator.prototype);
    case "Strong":
      return Object.setPrototypeOf(value, types.Strong.prototype);
    case "Styled":
      return Object.setPrototypeOf(value, types.Styled.prototype);
    case "StyledBlock":
      return Object.setPrototypeOf(value, types.StyledBlock.prototype);
    case "StyledInline":
      return Object.setPrototypeOf(value, types.StyledInline.prototype);
    case "Subscript":
      return Object.setPrototypeOf(value, types.Subscript.prototype);
    case "Suggestion":
      return Object.setPrototypeOf(value, types.Suggestion.prototype);
    case "SuggestionBlock":
      return Object.setPrototypeOf(value, types.SuggestionBlock.prototype);
    case "SuggestionInline":
      return Object.setPrototypeOf(value, types.SuggestionInline.prototype);
    case "Superscript":
      return Object.setPrototypeOf(value, types.Superscript.prototype);
    case "Table":
      return Object.setPrototypeOf(value, types.Table.prototype);
    case "TableCell":
      return Object.setPrototypeOf(value, types.TableCell.prototype);
    case "TableRow":
      return Object.setPrototypeOf(value, types.TableRow.prototype);
    case "Text":
      return Object.setPrototypeOf(value, types.Text.prototype);
    case "ThematicBreak":
      return Object.setPrototypeOf(value, types.ThematicBreak.prototype);
    case "Thing":
      return Object.setPrototypeOf(value, types.Thing.prototype);
    case "Time":
      return Object.setPrototypeOf(value, types.Time.prototype);
    case "TimeValidator":
      return Object.setPrototypeOf(value, types.TimeValidator.prototype);
    case "Timestamp":
      return Object.setPrototypeOf(value, types.Timestamp.prototype);
    case "TimestampValidator":
      return Object.setPrototypeOf(value, types.TimestampValidator.prototype);
    case "TupleValidator":
      return Object.setPrototypeOf(value, types.TupleValidator.prototype);
    case "Underline":
      return Object.setPrototypeOf(value, types.Underline.prototype);
    case "Unknown":
      return Object.setPrototypeOf(value, types.Unknown.prototype);
    case "Variable":
      return Object.setPrototypeOf(value, types.Variable.prototype);
    case "VideoObject":
      return Object.setPrototypeOf(value, types.VideoObject.prototype);
    case "Walkthrough":
      return Object.setPrototypeOf(value, types.Walkthrough.prototype);
    case "WalkthroughStep":
      return Object.setPrototypeOf(value, types.WalkthroughStep.prototype);
    // TYPE-CASES:STOP
    default:
      return value;
  }
}
