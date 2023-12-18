import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { defaultHighlightStyle, HighlightStyle } from '@codemirror/language'

import {
  StencilaColonSyntax,
  highlightStyles as cSyntaxStyles,
} from './extensions/colonSyntax'

customMarkdown = markdownLanguage.data.of({
  closeBrackets,
})

const markdownHighlightStyle = HighlightStyle.define([
  ...defaultHighlightStyle.specs,
  ...cSyntaxStyles,
])

const stencilaMarkdown = () =>
  markdown({
    base: markdownLanguage,
    extensions: [StencilaColonSyntax],
  })

export { stencilaMarkdown, markdownHighlightStyle }
