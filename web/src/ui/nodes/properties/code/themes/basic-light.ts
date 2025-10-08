import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Basic Light theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-basic-light
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/basic-light/src/index.ts
 */
export function basicLightTheme(): Extension {
  // Color palette
  const base00 = '#1c2434' // deep navy - primary text
  const base01 = '#2d3748' // dark slate - secondary text
  const base03 = '#718096' // steel blue - comments
  const base07 = '#0c7792' // teal - links, braces
  const base08 = '#0369a1' // azure blue - numbers
  const base09 = '#2b6cb0' // royal blue - variables
  const base0A = '#1a365d' // deep navy - keywords
  const base0B = '#c53030' // red - square brackets
  const base0C = '#dd6b20' // orange - strings
  const base0D = '#d69e2e' // amber - class names
  const base0E = '#2f855a' // green - operators
  const base0F = '#805ad5' // purple - tag names
  const invalid = '#e53e3e' // bright red - errors

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base0A, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base0A, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base0A, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base09 },
    { tag: [tags.variableName], color: base09 },
    { tag: [tags.propertyName], color: base09, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0D },
    { tag: [tags.className], color: base0D, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base09, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0E },
    { tag: [tags.bracket], color: base07 },
    { tag: [tags.brace], color: base07 },
    { tag: [tags.punctuation], color: base07 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base08 },
    { tag: [tags.definition(tags.variableName)], color: base09 },

    // Constants and literals
    { tag: tags.number, color: base08 },
    { tag: tags.changed, color: base08 },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base08, fontStyle: 'italic' },
    { tag: tags.self, color: base08 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0A },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base0C },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base07 },
    { tag: [tags.special(tags.string), tags.regexp], color: base0B },
    { tag: tags.string, color: base0C },

    // Comments and documentation
    { tag: tags.meta, color: base08 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0F },
    { tag: [tags.attributeName], color: base0D },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base08 },
    { tag: [tags.strong], fontWeight: 'bold', color: base09 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0A },

    // Links and URLs
    { tag: [tags.link], color: base07, fontWeight: '500', textDecoration: 'underline' },
    { tag: [tags.url], color: base0C, textDecoration: 'underline' },

    // Special states
    { tag: [tags.invalid], color: base00, textDecoration: 'underline wavy' },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Additional specific styles
    { tag: tags.squareBracket, color: base0B },
    { tag: tags.angleBracket, color: base0C },
    { tag: tags.monospace, color: base00 },
    { tag: [tags.contentSeparator], color: base0D },
    { tag: tags.quote, color: base01 },
  ])

  return syntaxHighlighting(highlightStyle)
}
