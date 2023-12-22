import { markdown, commonmarkLanguage } from '@codemirror/lang-markdown'
import {
  defaultHighlightStyle,
  HighlightStyle,
  foldNodeProp,
} from '@codemirror/language'
import {
  Table,
  TaskList,
  Autolink,
  Superscript,
  Subscript,
  Emoji,
} from '@lezer/markdown'

import {
  StencilaColonSyntax,
  highlightStyles as cSyntaxStyles,
} from './extensions/colonSyntax'
import { StencilaInlineStyle } from './extensions/inlineStyleSyntax'
import {
  StencilaInstructSyntax,
  highlightStyles as instSyntaxStyles,
} from './extensions/instructSyntax'
import {
  StencilaSuggestionSyntax,
  highlightStyles as suggSyntaxStyles,
} from './extensions/suggestionSyntax'

// choose the markdown extensions from @lezer/markdown
// to provide more customisation
const LezerMdExtensions = [
  Table,
  TaskList,
  Subscript,
  Superscript,
  Autolink,
  Emoji,
  {
    props: [
      foldNodeProp.add({
        Table: (tree, state) => ({
          from: state.doc.lineAt(tree.from).to,
          to: tree.to,
        }),
      }),
    ],
  },
]

const markdownHighlightStyle = HighlightStyle.define([
  ...cSyntaxStyles,
  ...suggSyntaxStyles,
  ...instSyntaxStyles,
  ...defaultHighlightStyle.specs,
])

/**
 * Creates a custom markdown `LanguageSupport` object,
 * using common markdown and selected lezer markdown extensions as the base.
 * Custom stencila syntax highlighting is added on to aswell.
 *
 * @returns codemirror 6 `LanguageSupport`
 */
const stencilaMarkdown = () =>
  markdown({
    base: commonmarkLanguage,
    extensions: [
      ...LezerMdExtensions,
      StencilaColonSyntax,
      StencilaInstructSyntax,
      StencilaSuggestionSyntax,
    ],
  })

export { stencilaMarkdown, markdownHighlightStyle }
