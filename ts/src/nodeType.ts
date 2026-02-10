// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

export type NodeType =
  | "Null"
  | "Boolean"
  | "Integer"
  | "UnsignedInteger"
  | "Number"
  | "String"
  | "Array"
  | "Admonition"
  | "Annotation"
  | "AppendixBreak"
  | "ArrayHint"
  | "ArrayValidator"
  | "Article"
  | "AudioObject"
  | "AuthorRole"
  | "Bibliography"
  | "BooleanValidator"
  | "Brand"
  | "Button"
  | "CallArgument"
  | "CallBlock"
  | "Chat"
  | "ChatMessage"
  | "ChatMessageGroup"
  | "Citation"
  | "CitationGroup"
  | "Claim"
  | "CodeBlock"
  | "CodeChunk"
  | "CodeExpression"
  | "CodeInline"
  | "CodeLocation"
  | "Collection"
  | "Comment"
  | "CompilationDigest"
  | "CompilationMessage"
  | "ConstantValidator"
  | "ContactPoint"
  | "CreativeWork"
  | "Datatable"
  | "DatatableColumn"
  | "DatatableColumnHint"
  | "DatatableHint"
  | "Date"
  | "DateTime"
  | "DateTimeValidator"
  | "DateValidator"
  | "DefinedTerm"
  | "Directory"
  | "Duration"
  | "DurationValidator"
  | "Emphasis"
  | "EnumValidator"
  | "Enumeration"
  | "Excerpt"
  | "ExecutionDependant"
  | "ExecutionDependency"
  | "ExecutionMessage"
  | "ExecutionTag"
  | "Figure"
  | "File"
  | "ForBlock"
  | "Form"
  | "Function"
  | "Grant"
  | "Heading"
  | "IfBlock"
  | "IfBlockClause"
  | "ImageObject"
  | "IncludeBlock"
  | "InlinesBlock"
  | "InstructionBlock"
  | "InstructionInline"
  | "InstructionMessage"
  | "IntegerValidator"
  | "Island"
  | "Link"
  | "List"
  | "ListItem"
  | "MathBlock"
  | "MathInline"
  | "MediaObject"
  | "ModelParameters"
  | "MonetaryGrant"
  | "Note"
  | "NumberValidator"
  | "ObjectHint"
  | "Organization"
  | "Page"
  | "Paragraph"
  | "Parameter"
  | "Periodical"
  | "Person"
  | "PostalAddress"
  | "Product"
  | "Prompt"
  | "PromptBlock"
  | "PropertyValue"
  | "ProvenanceCount"
  | "PublicationIssue"
  | "PublicationVolume"
  | "QuoteBlock"
  | "QuoteInline"
  | "RawBlock"
  | "Reference"
  | "Review"
  | "Section"
  | "Sentence"
  | "Skill"
  | "SoftwareApplication"
  | "SoftwareSourceCode"
  | "Strikeout"
  | "StringHint"
  | "StringValidator"
  | "Strong"
  | "StyledBlock"
  | "StyledInline"
  | "Subscript"
  | "SuggestionBlock"
  | "SuggestionInline"
  | "Superscript"
  | "Supplement"
  | "Table"
  | "TableCell"
  | "TableRow"
  | "Text"
  | "ThematicBreak"
  | "Thing"
  | "Time"
  | "TimeValidator"
  | "Timestamp"
  | "TimestampValidator"
  | "TupleValidator"
  | "Underline"
  | "Unknown"
  | "Variable"
  | "VideoObject"
  | "Walkthrough"
  | "WalkthroughStep"
  | "Cord"
  | "Object";

export const NodeTypeList = [
  "Null",
  "Boolean",
  "Integer",
  "UnsignedInteger",
  "Number",
  "String",
  "Array",
  "Admonition",
  "Annotation",
  "AppendixBreak",
  "ArrayHint",
  "ArrayValidator",
  "Article",
  "AudioObject",
  "AuthorRole",
  "Bibliography",
  "BooleanValidator",
  "Brand",
  "Button",
  "CallArgument",
  "CallBlock",
  "Chat",
  "ChatMessage",
  "ChatMessageGroup",
  "Citation",
  "CitationGroup",
  "Claim",
  "CodeBlock",
  "CodeChunk",
  "CodeExpression",
  "CodeInline",
  "CodeLocation",
  "Collection",
  "Comment",
  "CompilationDigest",
  "CompilationMessage",
  "ConstantValidator",
  "ContactPoint",
  "CreativeWork",
  "Datatable",
  "DatatableColumn",
  "DatatableColumnHint",
  "DatatableHint",
  "Date",
  "DateTime",
  "DateTimeValidator",
  "DateValidator",
  "DefinedTerm",
  "Directory",
  "Duration",
  "DurationValidator",
  "Emphasis",
  "EnumValidator",
  "Enumeration",
  "Excerpt",
  "ExecutionDependant",
  "ExecutionDependency",
  "ExecutionMessage",
  "ExecutionTag",
  "Figure",
  "File",
  "ForBlock",
  "Form",
  "Function",
  "Grant",
  "Heading",
  "IfBlock",
  "IfBlockClause",
  "ImageObject",
  "IncludeBlock",
  "InlinesBlock",
  "InstructionBlock",
  "InstructionInline",
  "InstructionMessage",
  "IntegerValidator",
  "Island",
  "Link",
  "List",
  "ListItem",
  "MathBlock",
  "MathInline",
  "MediaObject",
  "ModelParameters",
  "MonetaryGrant",
  "Note",
  "NumberValidator",
  "ObjectHint",
  "Organization",
  "Page",
  "Paragraph",
  "Parameter",
  "Periodical",
  "Person",
  "PostalAddress",
  "Product",
  "Prompt",
  "PromptBlock",
  "PropertyValue",
  "ProvenanceCount",
  "PublicationIssue",
  "PublicationVolume",
  "QuoteBlock",
  "QuoteInline",
  "RawBlock",
  "Reference",
  "Review",
  "Section",
  "Sentence",
  "Skill",
  "SoftwareApplication",
  "SoftwareSourceCode",
  "Strikeout",
  "StringHint",
  "StringValidator",
  "Strong",
  "StyledBlock",
  "StyledInline",
  "Subscript",
  "SuggestionBlock",
  "SuggestionInline",
  "Superscript",
  "Supplement",
  "Table",
  "TableCell",
  "TableRow",
  "Text",
  "ThematicBreak",
  "Thing",
  "Time",
  "TimeValidator",
  "Timestamp",
  "TimestampValidator",
  "TupleValidator",
  "Underline",
  "Unknown",
  "Variable",
  "VideoObject",
  "Walkthrough",
  "WalkthroughStep",
  "Cord",
  "Object",
];
