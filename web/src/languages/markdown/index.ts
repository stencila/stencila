import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { defaultHighlightStyle, HighlightStyle } from '@codemirror/language'

import {
  StencilaColonSyntax,
  highlightStyles as cSyntaxStyles,
} from './extensions/colonSyntax'
import {
  StencilaInstructSyntax,
  highlightStyles as instSyntaxStyles,
} from './extensions/instructSyntax'

const markdownHighlightStyle = HighlightStyle.define([
  ...defaultHighlightStyle.specs,
  ...cSyntaxStyles,
  ...instSyntaxStyles,
])

const stencilaMarkdown = () =>
  markdown({
    base: markdownLanguage,
    extensions: [StencilaColonSyntax, StencilaInstructSyntax],
  })

export { stencilaMarkdown, markdownHighlightStyle }
