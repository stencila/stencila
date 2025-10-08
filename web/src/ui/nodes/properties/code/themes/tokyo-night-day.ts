import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Tokyo Night Day theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-tokyo-night-day
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/tokyo-night-day/src/index.ts
 */
export function tokyoNightDayTheme(): Extension {
  // Color palette
  const base01 = '#3760bf' // Primary foreground
  const base03 = '#848cb5' // Comments
  const base_red = '#f52a65' // Errors
  const base_orange = '#b15c00' // Numbers
  const base_yellow = '#8c6c3e' // Classes
  const base_green = '#587539' // Strings
  const base_cyan = '#007197' // Functions, keywords
  const base_blue = '#2e7de9' // Variables
  const base_purple = '#7847bd' // Operators, tags
  const base_magenta = '#9854f1' // Special
  const invalid = base_red

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base_cyan, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base_cyan, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base_cyan, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base_blue },
    { tag: [tags.variableName], color: base01 },
    { tag: [tags.propertyName], color: base_blue, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base_cyan },
    { tag: [tags.className], color: base_yellow, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base_purple, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base_purple },
    { tag: [tags.bracket], color: base03 },
    { tag: [tags.brace], color: base03 },
    { tag: [tags.punctuation], color: base03 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName)], color: base_cyan },
    { tag: [tags.labelName], color: base_purple, fontStyle: 'italic' },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base_cyan },
    { tag: [tags.definition(tags.variableName)], color: base_blue },

    // Constants and literals
    { tag: tags.number, color: base_orange },
    { tag: tags.changed, color: base_orange },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base_orange, fontStyle: 'italic' },
    { tag: tags.self, color: base_orange },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base_orange },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base_orange },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base_green },
    { tag: [tags.special(tags.string), tags.regexp], color: base_magenta },
    { tag: tags.string, color: base_green },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base_cyan, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base_blue },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base_purple },
    { tag: [tags.attributeName], color: base_yellow },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base_orange },
    { tag: tags.heading1, color: base_orange, fontWeight: 'bold' },
    { tag: tags.heading2, color: base_orange },
    { tag: tags.heading3, color: base_orange },
    { tag: tags.heading4, color: base_cyan },
    { tag: tags.heading5, color: base_cyan },
    { tag: tags.heading6, color: base_cyan },
    { tag: [tags.strong], fontWeight: 'bold', color: base01 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base_green },

    // Links and URLs
    { tag: [tags.link], color: base_purple, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: base_cyan, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base01, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base_orange },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base03 },
  ])

  return syntaxHighlighting(highlightStyle)
}
