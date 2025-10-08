import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Tokyo Night Storm theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-tokyo-night-storm
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/tokyo-night-storm/src/index.ts
 */
export function tokyoNightStormTheme(): Extension {
  // Color palette
  const base03 = '#565f89' // Comments
  const base04 = '#c0caf5' // Default foreground
  const base08 = '#ff9e64' // Numbers, constants
  const base09 = '#e0af68' // Classes, attributes
  const base0A = '#9ece6a' // Strings
  const base0B = '#2ac3de' // Types, parameter
  const base0C = '#7aa2f7' // Functions, properties
  const base0D = '#bb9af7' // Keywords, operators
  const invalid = '#f7768e'

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base0D, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base0D, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base0D, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base04 },
    { tag: [tags.variableName], color: base04 },
    { tag: [tags.propertyName], color: base0C, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0B },
    { tag: [tags.className], color: base09, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0C, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0D },
    { tag: [tags.bracket], color: base03 },
    { tag: [tags.brace], color: base03 },
    { tag: [tags.punctuation], color: base03 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName)], color: base0C },
    { tag: [tags.labelName], color: base0C, fontStyle: 'italic' },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base0C },
    { tag: [tags.definition(tags.variableName)], color: base04 },

    // Constants and literals
    { tag: tags.number, color: base08 },
    { tag: tags.changed, color: base08 },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base08, fontStyle: 'italic' },
    { tag: tags.self, color: base08 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0D },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base08 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0A },
    { tag: [tags.special(tags.string), tags.regexp], color: '#b4f9f8' },
    { tag: tags.string, color: base0A },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0B, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base04 },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0D },
    { tag: [tags.attributeName], color: base09 },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: '#89ddff' },
    { tag: tags.heading1, color: '#89ddff', fontWeight: 'bold' },
    { tag: tags.heading2, color: '#89ddff' },
    { tag: tags.heading3, color: '#89ddff' },
    { tag: tags.heading4, color: '#89ddff' },
    { tag: tags.heading5, color: '#89ddff' },
    { tag: tags.heading6, color: '#89ddff' },
    { tag: [tags.strong], fontWeight: 'bold', color: base04 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0B },

    // Links and URLs
    { tag: [tags.link], color: base0D, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: base0C, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base04, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0D },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base03 },
  ])

  return syntaxHighlighting(highlightStyle)
}
