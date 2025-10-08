import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Abyss theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-abyss
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/abyss/src/index.ts
 */
export function abyssTheme(): Extension {
  // Color palette
  const base03 = '#b4a2f7' // Gutter/types
  const base07 = '#47c1ff' // Keywords (brighter cyan blue)
  const base08 = '#5caeff' // Variables (softer azure blue)
  const base09 = '#7599c2' // Comments (brighter blue-gray)
  const base0A = '#4ce660' // Strings (vibrant green)
  const base0B = '#c3a2f7' // Functions (softer purple)
  const base0C = '#ff9eea' // Constants (softer pink)
  const base0D = '#ffd47b' // Classes (warmer gold)
  const base0E = '#8eb8ff' // Headings (brighter sky blue)
  const base0F = '#59d6ff' // Tags (brighter cyan)
  const base10 = '#ff50c8' // Links (brighter magenta)
  const base11 = '#66ecd4' // URLs (brighter teal)
  const invalid = '#ff3333' // Error color

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base07, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base0F, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base07, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base08 },
    { tag: [tags.variableName], color: '#7ab2ff' },
    { tag: [tags.propertyName], color: base0E, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base03 },
    { tag: [tags.className], color: base0D, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0C, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: '#78b6ff' },
    { tag: [tags.bracket], color: '#8da0bf' },
    { tag: [tags.brace], color: '#8da0bf' },
    { tag: [tags.punctuation], color: '#8da0bf' },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base0B },
    { tag: [tags.definition(tags.variableName)], color: base0B },

    // Constants and literals
    { tag: tags.number, color: base0C },
    { tag: tags.changed, color: base0C },
    { tag: tags.annotation, color: base0C, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0C, fontStyle: 'italic' },
    { tag: tags.self, color: base0C },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0C },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: '#ff9e64' },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0A },
    { tag: [tags.special(tags.string), tags.regexp], color: base0A },
    { tag: tags.string, color: base0A },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0B, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base09 },
    { tag: tags.comment, fontStyle: 'italic', color: base09 },
    { tag: tags.docComment, fontStyle: 'italic', color: base09 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0F },
    { tag: [tags.attributeName], color: '#ffd580' },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0E },
    { tag: [tags.strong], fontWeight: 'bold' },
    { tag: [tags.emphasis], fontStyle: 'italic' },

    // Links and URLs
    { tag: [tags.link], color: base10, fontWeight: '500' },
    { tag: [tags.url], color: base11, textDecoration: 'underline' },

    // Special states
    { tag: [tags.invalid], color: invalid, textDecoration: 'underline wavy' },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },
  ])

  return syntaxHighlighting(highlightStyle)
}
