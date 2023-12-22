import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { defaultHighlightStyle, HighlightStyle } from '@codemirror/language'

import {
  StencilaColonSyntax,
  highlightStyles as cSyntaxStyles,
} from './extensions/colonSyntax'
import { StencilaInlineStyle } from './extensions/inlineStyleSyntax'

// const stencilaMarkdownExtension: MarkdownConfig = {
//   defineNodes: [...ifBlockNodeList],
//   parseBlock: [...ifParsers]
// }

const markdownHighlightStyle = HighlightStyle.define([
  ...defaultHighlightStyle.specs,
  ...cSyntaxStyles,
])

const stencilaMarkdown = () =>
  markdown({
    base: markdownLanguage,
    extensions: [StencilaColonSyntax, StencilaInlineStyle],
  })

export { stencilaMarkdown, markdownHighlightStyle }
