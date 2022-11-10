/**
 * ProseMirror schema for a Stencila nodes
 *
 * This schema uses the following conventions:
 *
 * - Stencila node types are represented as ProseMirror `NodeSpec`s with title
 *   case name (e.g. `Paragraph`).
 *
 * - Properties of Stencila nodes, containing inline or block content, other than
 *   the `content` property, are represented as ProseMirror `NodeSpec`s with a lowercase
 *   name (e.g. Article `title` and `description`).
 *
 * - Stencila node types can define a `contentProp` which is the name of the node property
 *   that will be used when generating an address e.g. `contentProp: 'cells'`
 *
 * These conventions make it possible to convert a ProseMirror offset position e.g. `83`
 * into a Stencila address e.g. `["content", 1, "caption", 4]`.
 *
 * Note: When adding types here, please ensure transformations are handled in the
 * `transformProsemirror` function.
 *
 * For docs and examples see:
 *  - https://prosemirror.net/docs/guide/#schema
 *  - https://prosemirror.net/examples/schema/
 *  - https://github.com/ProseMirror/prosemirror-schema-basic/blob/master/src/schema-basic.js
 */

import { Node, Schema } from 'prosemirror-model'
import { EditorView } from 'prosemirror-view'

import { article } from './article'
import { button, StencilaButtonView } from './button'
import { call, StencilaCallView } from './call'
import { codeBlock, StencilaCodeBlockView } from './code-block'
import { codeChunk, StencilaCodeChunkView } from './code-chunk'
import { codeExpression, StencilaCodeExpressionView } from './code-expression'
import { codeFragment, StencilaCodeFragmentView } from './code-fragment'
import { division, StencilaDivisionView } from './division'
import { for_, StencilaForView } from './for'
import { form, StencilaFormView } from './form'
import { heading } from './heading'
import { if_, StencilaIfView } from './if'
import { ifClause, StencilaIfClauseView } from './if-clause'
import { include, StencilaIncludeView } from './include'
import { list, listItem } from './list'
import {
  emphasis,
  quote,
  strikeout,
  strong,
  subscript,
  superscript,
  underline,
} from './marks'
import { mathBlock, StencilaMathBlockView } from './math-block'
import { mathFragment, StencilaMathFragmentView } from './math-fragment'
import { paragraph } from './paragraph'
import { parameter, StencilaParameterView } from './parameter'
import { quoteBlock } from './quote-block'
import { span, StencilaSpanView } from './span'
import { table, tableCell, tableHeader, tableRow } from './table'
import { thematicBreak } from './thematic-break'

const blocks = {
  // Paragraph should be first since it is the default
  // block content i.e. used for BlockContent+ expressions
  Paragraph: paragraph(),

  Call: call(),
  CodeBlock: codeBlock(),
  CodeChunk: codeChunk(),
  Division: division(),
  For: for_(),
  Form: form(),
  Heading: heading(),
  If: if_(),
  IfClause: ifClause(),
  Include: include(),
  List: list(),
  ListItem: listItem(),
  MathBlock: mathBlock(),
  QuoteBlock: quoteBlock(),
  Table: table(),
  TableRow: tableRow(),
  TableCell: tableCell(),
  TableHeader: tableHeader(),
  ThematicBreak: thematicBreak(),
}

const inlines = {
  Button: button(),
  CodeExpression: codeExpression(),
  CodeFragment: codeFragment(),
  MathFragment: mathFragment(),
  Parameter: parameter(),
  Span: span(),
  text: { group: 'InlineContent' },
}

const marks = {
  Emphasis: emphasis(),
  Quote: quote(),
  Strikeout: strikeout(),
  Strong: strong(),
  Subscript: subscript(),
  Superscript: superscript(),
  Underline: underline(),
}

export const articleSchema = new Schema({
  topNode: 'Article',
  nodes: {
    ...article,
    ...blocks,
    ...inlines,
  },
  marks,
})

export const nodeViews = {
  Button: nodeView(StencilaButtonView),
  Call: nodeView(StencilaCallView),
  CodeBlock: nodeView(StencilaCodeBlockView),
  CodeChunk: nodeView(StencilaCodeChunkView),
  CodeExpression: nodeView(StencilaCodeExpressionView),
  CodeFragment: nodeView(StencilaCodeFragmentView),
  Division: nodeView(StencilaDivisionView),
  For: nodeView(StencilaForView),
  Form: nodeView(StencilaFormView),
  If: nodeView(StencilaIfView),
  IfClause: nodeView(StencilaIfClauseView),
  Include: nodeView(StencilaIncludeView),
  MathBlock: nodeView(StencilaMathBlockView),
  MathFragment: nodeView(StencilaMathFragmentView),
  Parameter: nodeView(StencilaParameterView),
  Span: nodeView(StencilaSpanView),
}

function nodeView(Type) {
  return (node: Node, view: EditorView, getPos: () => number) => {
    return new Type(node, view, getPos)
  }
}
