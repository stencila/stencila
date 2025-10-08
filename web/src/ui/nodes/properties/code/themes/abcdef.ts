import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Abcdef theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-abcdef
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/abcdef/src/index.ts
 */
export function abcdefTheme(): Extension {
  // Color palette
  const base08 = '#efbb24' // Keyword (warm gold)
  const base09 = '#7799ff' // Atom (brighter blue)
  const base0A = '#8c8f93' // Comment (slightly bluer gray)
  const base0B = '#c792ea' // Number (softer purple)
  const base0C = '#ffee99' // Definition, Function (softer yellow)
  const base0D = '#abcdef' // Variable (theme's signature color)
  const base0E = '#ffcc44' // Type Name (warmer yellow)
  const base0F = '#99c2ff' // Tag Name (light blue)
  const invalid = '#ff3333' // Error color

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base08, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base08, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base08, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base09 },
    { tag: [tags.variableName], color: base09 },
    { tag: [tags.propertyName], color: base0D, fontStyle: 'normal' },

    // Classes and types
    { tag: tags.typeName, color: base0E },
    { tag: tags.className, color: base0D, fontStyle: 'italic' },
    { tag: tags.namespace, color: '#78a0d3' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: '#ff9cac' },
    { tag: [tags.bracket], color: '#d0d6e0' },
    { tag: [tags.brace], color: '#d0d6e0' },
    { tag: [tags.punctuation], color: '#d0d6e0' },

    // Functions and parameters
    { tag: tags.function(tags.variableName), color: base0C },
    { tag: tags.definition(tags.variableName), color: base0C },

    // Constants and literals
    { tag: tags.number, color: base0B },
    { tag: tags.changed, color: '#ff9cac' },
    { tag: tags.annotation, color: '#ffad5c', fontStyle: 'italic' },
    { tag: tags.modifier, color: '#ffad5c', fontStyle: 'italic' },
    { tag: tags.self, color: base0B },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0B },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base0B },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: '#7aecb3' },
    { tag: [tags.special(tags.string), tags.regexp], color: '#6acdbe' },
    { tag: tags.string, color: '#7aecb3' },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0E, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: '#78a0d3' },
    { tag: tags.comment, fontStyle: 'italic', color: base0A },
    { tag: tags.docComment, fontStyle: 'italic', color: base0A },

    // HTML/XML elements
    { tag: tags.tagName, color: base0F },
    { tag: tags.attributeName, color: '#ffad5c' },

    // Markdown and text formatting
    { tag: tags.heading, color: base0E, fontWeight: 'bold' },
    { tag: [tags.strong], fontWeight: 'bold' },
    { tag: [tags.emphasis], fontStyle: 'italic' },

    // Links and URLs
    { tag: tags.link, color: '#ffcc44', fontWeight: '500' },
    { tag: tags.url, color: '#66c2ff', textDecoration: 'underline' },

    // Special states
    { tag: tags.invalid, color: invalid, textDecoration: 'underline wavy' },
    { tag: tags.strikethrough, color: invalid, textDecoration: 'line-through' },

    // Additional specific styles
    { tag: tags.labelName, color: '#ffad5c' },
  ])

  return syntaxHighlighting(highlightStyle)
}
