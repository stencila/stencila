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
import { heading } from './heading'
import { if_, StencilaIfView } from './if'
import { ifClause, StencilaIfClauseView } from './if-clause'
import { include, StencilaIncludeView } from './include'
import {
  emphasis,
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
import { span, StencilaSpanView } from './span'

const blocks = {
  // Paragraph should be first since it is the default
  // block content i.e. used for BlockContent+ expressions
  Paragraph: paragraph(),

  Call: call(),
  CodeBlock: codeBlock(),
  CodeChunk: codeChunk(),
  Division: division(),
  For: for_(),
  Heading: heading(),
  If: if_(),
  IfClause: ifClause(),
  Include: include(),
  MathBlock: mathBlock(),
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
