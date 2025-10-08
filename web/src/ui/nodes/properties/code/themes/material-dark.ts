import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Material Dark theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-material-dark
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/material-dark/src/index.ts
 */
export function materialDarkTheme(): Extension {
  // Color palette
  const base03 = '#707d8b' // Comments
  const base05 = '#bdbdbd' // Default foreground
  const base08 = '#ff5f52' // Keywords
  const base09 = '#ff6e40' // Constants
  const base0A = '#fa5788' // Regex, special
  const base0B = '#facf4e' // Classes, numbers
  const base0C = '#ffad42' // Strings
  const base0D = '#56c8d8' // Functions
  const base0E = '#7186f0' // Variables, operators
  const base0F = '#cf6edf' // Tags
  const base10 = '#6abf69' // Added elements
  const base11 = '#99d066' // Modified elements
  const base12 = '#4ebaaa' // Headings
  const invalid = base08
  const linkColor = base0D
  const visitedLinkColor = base0F

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base08, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base08, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base08, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base0E },
    { tag: [tags.variableName], color: base11 },
    { tag: [tags.propertyName], color: base0F, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base08 },
    { tag: [tags.className], color: base0B, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0E, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base05 },
    { tag: [tags.bracket], color: base03 },
    { tag: [tags.brace], color: base03 },
    { tag: [tags.punctuation], color: base03 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName)], color: base0D },
    { tag: [tags.labelName], color: base0D, fontStyle: 'italic' },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base0D },
    { tag: [tags.definition(tags.variableName)], color: base0E },

    // Constants and literals
    { tag: tags.number, color: base0B },
    { tag: tags.changed, color: base0B },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base09, fontStyle: 'italic' },
    { tag: tags.self, color: base09 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base09 },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base09 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base10 },
    { tag: [tags.special(tags.string), tags.regexp], color: base0A },
    { tag: tags.string, color: base0C },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base08, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0F },
    { tag: [tags.attributeName], color: base0B },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base12 },
    { tag: tags.heading1, color: base0B },
    { tag: tags.heading2, color: base0C },
    { tag: tags.heading3, color: base0D },
    { tag: tags.heading4, color: base0E },
    { tag: tags.heading5, color: base0F },
    { tag: tags.heading6, color: base08 },
    { tag: [tags.strong], fontWeight: 'bold', color: base0E },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0C },

    // Links and URLs
    { tag: [tags.link], color: visitedLinkColor, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: linkColor, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base05, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base09 },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base03 },
  ])

  return syntaxHighlighting(highlightStyle)
}
