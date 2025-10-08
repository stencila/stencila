import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Solarized Light theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-solarized-light
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/solarized-light/src/index.ts
 */
export function solarizedLightTheme(): Extension {
  // Color palette
  const base00 = '#657b83' // Body text
  const base01 = '#586e75' // Comments
  const base02 = '#073642' // Background highlights
  const base03 = '#002b36' // Comments
  const base09 = '#dc322f' // Red
  const base0A = '#cb4b16' // Orange
  const base0B = '#b58900' // Yellow
  const base0C = '#859900' // Green
  const base0D = '#2aa198' // Cyan
  const base0E = '#268bd2' // Blue
  const base0F = '#6c71c4' // Violet
  const base10 = '#d33682' // Magenta
  const invalid = '#d30102'

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base0C, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base0C, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base0C, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base0D },
    { tag: [tags.variableName], color: base00 },
    { tag: [tags.propertyName], color: base0D, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0A },
    { tag: [tags.className], color: base0A, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base10, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0F },
    { tag: [tags.bracket], color: base10 },
    { tag: [tags.brace], color: base10 },
    { tag: [tags.punctuation], color: base01 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName)], color: base0E },
    { tag: [tags.labelName], color: base10 },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base0E },
    { tag: [tags.definition(tags.variableName)], color: base0D },

    // Constants and literals
    { tag: tags.number, color: base10 },
    { tag: tags.changed, color: base10 },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base10, fontStyle: 'italic' },
    { tag: tags.self, color: base10 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0B },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base10 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0C },
    { tag: [tags.special(tags.string), tags.regexp], color: invalid },
    { tag: tags.string, color: base0B },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0A, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base0D },

    // Comments and documentation
    { tag: tags.meta, color: base09 },
    { tag: tags.comment, fontStyle: 'italic', color: base01 },
    { tag: tags.docComment, fontStyle: 'italic', color: base01 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0E },
    { tag: [tags.attributeName], color: base00 },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0B },
    { tag: tags.heading1, color: base03 },
    { tag: tags.heading2, color: base02 },
    { tag: tags.heading3, color: base02 },
    { tag: tags.heading4, color: base02 },
    { tag: tags.heading5, color: base02 },
    { tag: tags.heading6, color: base02 },
    { tag: [tags.strong], fontWeight: 'bold', color: base02 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0C },

    // Links and URLs
    { tag: [tags.link], color: base0E, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: base0E, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base00, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0B },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base10 },
  ])

  return syntaxHighlighting(highlightStyle)
}
