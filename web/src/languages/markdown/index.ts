import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { defaultHighlightStyle, HighlightStyle } from '@codemirror/language'
import { Table } from '@lezer/markdown'

import {
  StencilaColonSyntax,
  highlightStyles as cSyntaxStyles,
} from './extensions/colonSyntax'
import {
  StencilaInstructSyntax,
  highlightStyles as instSyntaxStyles,
} from './extensions/instructSyntax'
import {
  StencilaSuggestionSyntax,
  highlightStyles as suggSyntaxStyles,
} from './extensions/suggestionSyntax'

const markdownHighlightStyle = HighlightStyle.define([
  ...cSyntaxStyles,
  ...suggSyntaxStyles,
  ...instSyntaxStyles,
  ...defaultHighlightStyle.specs,
])

const stencilaMarkdown = () =>
  markdown({
    base: markdownLanguage,
    extensions: [
      Table,
      StencilaColonSyntax,
      StencilaInstructSyntax,
      StencilaSuggestionSyntax,
    ],
  })

export { stencilaMarkdown, markdownHighlightStyle }
