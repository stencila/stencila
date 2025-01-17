import { NodeType } from '@stencila/types'

/**
 * Mapping object to match the stencila `NodeType` with it's respective custom element `tagName` (lowercase)
 */
export const tagNameToNodeTypeMap: { [k: string]: NodeType } = {
  'stencila-admonition': 'Admonition',
  'stencila-array': 'Array',
  'stencila-article': 'Article',
  'stencila-boolean': 'Boolean',
  'stencila-call-block': 'CallBlock',
  'stencila-claim': 'Claim',
  'stencila-code-block': 'CodeBlock',
  'stencila-code-chunk': 'CodeChunk',
  'stencila-code-inline': 'CodeInline',
  'stencila-code-expression': 'CodeExpression',
  'stencila-datatable': 'Datatable',
  'stencila-figure': 'Figure',
  'stencila-for-block': 'ForBlock',
  'stencila-heading': 'Heading',
  'stencila-if-block': 'IfBlock',
  'stencila-image-object': 'ImageObject',
  'stencila-instruction-block': 'InstructionBlock',
  'stencila-instruction-inline': 'InstructionInline',
  'stencila-integer': 'Integer',
  'stencila-list': 'List',
  'stencila-math-block': 'MathBlock',
  'stencila-math-inline': 'MathInline',
  'stencila-number': 'Number',
  'stencila-paragraph': 'Paragraph',
  'stencila-object': 'Object',
  'stencila-parameter': 'Parameter',
  'stencila-quote-block': 'QuoteBlock',
  'stencila-raw-block': 'RawBlock',
  'stencila-section': 'Section',
  'stencila-string': 'String',
  'stencila-styled-block': 'StyledBlock',
  'stencila-styled-inline': 'StyledInline',
  'stencila-table': 'Table',
  'stencila-text': 'Text',
  'stencila-unsigned-integer': 'UnsignedInteger',
}

/**
 * Mapping object to fetch the custom element `tagName` (lowercase) with it's respective stencila `NodeType`
 */
const nodeTypeToTagNameMap: { [k in NodeType]?: string } = {
  Admonition: 'stencila-admonition',
  Array: 'stencila-array',
  Article: 'stencila-article',
  Boolean: 'stencila-boolean',
  CallBlock: 'stencila-call-block',
  Claim: 'stencila-claim',
  CodeBlock: 'stencila-code-block',
  CodeChunk: 'stencila-code-chunk',
  CodeInline: 'stencila-code-inline',
  CodeExpression: 'stencila-code-expression',
  Datatable: 'stencila-datatable',
  Figure: 'stencila-figure',
  ForBlock: 'stencila-for-block',
  Heading: 'stencila-heading',
  IfBlock: 'stencila-if-block',
  ImageObject: 'stencila-image-object',
  InstructionBlock: 'stencila-instruction-block',
  InstructionInline: 'stencila-instruction-inline',
  Integer: 'stencila-integer',
  List: 'stencila-list',
  MathBlock: 'stencial-math-block',
  MathInline: 'stencila-math-inline',
  Number: 'stencila-number',
  Paragraph: 'stencila-paragraph',
  Object: 'stencila-object',
  Parameter: 'stencila-parameter',
  QuoteBlock: 'stencila-quote-block',
  RawBlock: 'stencila-raw-block',
  Section: 'stencila-section',
  String: 'stencila-string',
  StyledBlock: 'stencila-styled-block',
  StyledInline: 'stencila-styled-inline',
  Table: 'stencila-table',
  Text: 'stencila-text',
  UnsignedInteger: 'stencila-unsigned-integer',
}

export const nodeTypeToTagName = (node: NodeType): string => {
  return nodeTypeToTagNameMap[node] ?? 'stencila-text'
}

export const tagNameToNodeType = (tag: string): NodeType => {
  return tagNameToNodeTypeMap[tag] ?? 'Null'
}
