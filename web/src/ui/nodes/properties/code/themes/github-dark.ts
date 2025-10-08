import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * GitHub Dark theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-github-dark
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/github-dark/src/index.ts
 */
export function githubDarkTheme(): Extension {
  // Color palette
  const base01 = '#c9d1d9' // Foreground
  const base03 = '#8b949e' // Comment and Bracket color
  const base05 = '#7ee787' // TagName, Name, Quote
  const base06 = '#d2a8ff' // ClassName, PropertyName
  const base07 = '#79c0ff' // VariableName, Number
  const base08 = '#ff7b72' // Keyword, TypeName
  const base09 = '#a5d6ff' // String, Meta, Regexp
  const base0C = '#ffab70' // Atom, Bool
  const invalid = '#f97583' // Invalid color
  const linkColor = '#58a6ff' // Bright blue for links
  const visitedLinkColor = '#bc8cff' // Light purple for visited links

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base08, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base08, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base08, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base07 },
    { tag: [tags.variableName], color: base07 },
    { tag: [tags.propertyName], color: base06, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base08 },
    { tag: [tags.className], color: base06, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base07, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base01 },
    { tag: [tags.bracket], color: base03 },
    { tag: [tags.brace], color: base03 },
    { tag: [tags.punctuation], color: base03 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base05 },
    { tag: [tags.definition(tags.variableName)], color: base07 },

    // Constants and literals
    { tag: tags.number, color: base0C },
    { tag: tags.changed, color: base0C },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0C, fontStyle: 'italic' },
    { tag: tags.self, color: base0C },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0C },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base0C },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base05 },
    { tag: [tags.special(tags.string), tags.regexp], color: base09 },
    { tag: tags.string, color: base09 },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base08, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base05 },
    { tag: [tags.attributeName], color: base06 },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base07 },
    { tag: [tags.strong], fontWeight: 'bold', color: base07 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base09 },

    // Links and URLs
    { tag: [tags.link], color: visitedLinkColor, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: linkColor, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base01, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0C },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base03 },
  ])

  return syntaxHighlighting(highlightStyle)
}
