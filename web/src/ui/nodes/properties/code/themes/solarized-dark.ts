import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Solarized Dark theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-solarized-dark
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/solarized-dark/src/index.ts
 */
export function solarizedDarkTheme(): Extension {
  // Color palette
  const base02 = '#586e75' // Comments
  const base04 = '#839496' // Body text
  const base05 = '#93a1a1' // Default foreground
  const base06 = '#eee8d5' // Light foreground
  const base07 = '#fdf6e3' // Light background
  const base08 = '#dc322f' // Red
  const base09 = '#cb4b16' // Orange
  const base0A = '#b58900' // Yellow
  const base0B = '#859900' // Green
  const base0C = '#2aa198' // Cyan
  const base0D = '#268bd2' // Blue
  const base0E = '#6c71c4' // Violet
  const base0F = '#d33682' // Magenta
  const invalid = '#d30102'

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base0B, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base0B, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base0B, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base0C },
    { tag: [tags.variableName], color: base05 },
    { tag: [tags.propertyName], color: base0C, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base09 },
    { tag: [tags.className], color: base09, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0F, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0E },
    { tag: [tags.bracket], color: base0F },
    { tag: [tags.brace], color: base0F },
    { tag: [tags.punctuation], color: base04 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName)], color: base0D },
    { tag: [tags.labelName], color: base0F },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base0D },
    { tag: [tags.definition(tags.variableName)], color: base0C },

    // Constants and literals
    { tag: tags.number, color: base0F },
    { tag: tags.changed, color: base0F },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0F, fontStyle: 'italic' },
    { tag: tags.self, color: base0F },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0A },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base0F },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0B },
    { tag: [tags.special(tags.string), tags.regexp], color: invalid },
    { tag: tags.string, color: base0A },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base09, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base0C },

    // Comments and documentation
    { tag: tags.meta, color: base08 },
    { tag: tags.comment, fontStyle: 'italic', color: base02 },
    { tag: tags.docComment, fontStyle: 'italic', color: base02 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0D },
    { tag: [tags.attributeName], color: base05 },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0A },
    { tag: tags.heading1, color: base07 },
    { tag: tags.heading2, color: base06 },
    { tag: tags.heading3, color: base06 },
    { tag: tags.heading4, color: base06 },
    { tag: tags.heading5, color: base06 },
    { tag: tags.heading6, color: base06 },
    { tag: [tags.strong], fontWeight: 'bold', color: base06 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0B },

    // Links and URLs
    { tag: [tags.link], color: base0D, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: base0D, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base05, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0A },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base0F },
  ])

  return syntaxHighlighting(highlightStyle)
}
