import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Volcano theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-volcano
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/volcano/src/index.ts
 */
export function volcanoTheme(): Extension {
  // Color palette
  const base01 = '#F8F8F8' // Foreground
  const base03 = '#e7c0c0' // Comments
  const base05 = '#f12727' // Keywords
  const base08 = '#ec0d1e' // Errors, tags
  const base09 = '#aa5507' // Numbers
  const base0A = '#fec758' // Classes, variables
  const base0B = '#9df39f' // Success
  const base0C = '#7df3f7' // Functions
  const base0D = '#7dcaf7' // Variables
  const base0E = '#c27df7' // Operators
  const base0F = '#f77dca' // Special
  const invalid = '#ffffff'

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base05, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base05, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base05, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base01 },
    { tag: [tags.variableName], color: base0A },
    { tag: [tags.propertyName], color: base0C, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0B },
    { tag: [tags.className], color: base0A, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0D, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0E },
    { tag: [tags.bracket], color: base03 },
    { tag: [tags.brace], color: base03 },
    { tag: [tags.punctuation], color: base03 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName)], color: base0C },
    { tag: [tags.labelName], color: base0C, fontStyle: 'italic' },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base0C },
    { tag: [tags.definition(tags.variableName)], color: base0A },

    // Constants and literals
    { tag: tags.number, color: base09 },
    { tag: tags.changed, color: base09 },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base09, fontStyle: 'italic' },
    { tag: tags.self, color: base09 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base09 },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base09 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0B },
    { tag: [tags.special(tags.string), tags.regexp], color: base0F },
    { tag: tags.string, color: base0A },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0B, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base01 },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base08 },
    { tag: [tags.attributeName], color: base0A },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base09 },
    { tag: tags.heading1, color: base08, fontWeight: 'bold' },
    { tag: tags.heading2, color: base09 },
    { tag: tags.heading3, color: base0A },
    { tag: tags.heading4, color: base0B },
    { tag: tags.heading5, color: base0C },
    { tag: tags.heading6, color: base0D },
    { tag: [tags.strong], fontWeight: 'bold', color: base01 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0A },

    // Links and URLs
    { tag: [tags.link], color: base0E, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: base0C, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base01, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base09 },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base03 },
  ])

  return syntaxHighlighting(highlightStyle)
}
