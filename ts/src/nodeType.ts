// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

export type NodeType =
  | "Null"
  | "Boolean"
  | "Integer"
  | "UnsignedInteger"
  | "Number"
  | "String"
  | "Array"
  | "Action"
  | "Admonition"
  | "Agent"
  | "Annotation"
  | "AppendixBreak"
  | "ArrayHint"
  | "ArrayValidator"
  | "Article"
  | "AudioObject"
  | "AuthorRole"
  | "Bibliography"
  | "BooleanValidator"
  | "Boundary"
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
  | "ContainerImage"
  | "ConvertAction"
  | "CreateAction"
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
  | "Evidence"
  | "Excerpt"
  | "ExecuteAction"
  | "ExecutionMessage"
  | "ExecutionTag"
  | "Figure"
  | "File"
  | "ForBlock"
  | "Form"
  | "Function"
  | "Grant"
  | "Graph"
  | "GraphEdge"
  | "GraphEvidence"
  | "GraphNode"
  | "Heading"
  | "Icon"
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
  | "Protocol"
  | "ProvenanceCount"
  | "PublicationIssue"
  | "PublicationVolume"
  | "Question"
  | "QuoteBlock"
  | "QuoteInline"
  | "RawBlock"
  | "Reference"
  | "Request"
  | "ResearchObjectRelation"
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
  | "SymbolicLink"
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
  | "Workflow"
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
  "Action",
  "Admonition",
  "Agent",
  "Annotation",
  "AppendixBreak",
  "ArrayHint",
  "ArrayValidator",
  "Article",
  "AudioObject",
  "AuthorRole",
  "Bibliography",
  "BooleanValidator",
  "Boundary",
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
  "ContainerImage",
  "ConvertAction",
  "CreateAction",
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
  "Evidence",
  "Excerpt",
  "ExecuteAction",
  "ExecutionMessage",
  "ExecutionTag",
  "Figure",
  "File",
  "ForBlock",
  "Form",
  "Function",
  "Grant",
  "Graph",
  "GraphEdge",
  "GraphEvidence",
  "GraphNode",
  "Heading",
  "Icon",
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
  "Protocol",
  "ProvenanceCount",
  "PublicationIssue",
  "PublicationVolume",
  "Question",
  "QuoteBlock",
  "QuoteInline",
  "RawBlock",
  "Reference",
  "Request",
  "ResearchObjectRelation",
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
  "SymbolicLink",
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
  "Workflow",
  "Cord",
  "Object",
];
