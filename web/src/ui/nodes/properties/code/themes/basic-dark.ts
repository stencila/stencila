import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Basic Dark theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-basic-dark
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/basic-dark/src/index.ts
 */
export function basicDarkTheme(): Extension {
  // Color palette
  const base01 = '#e2e2e2' // Foreground
  const base06 = '#909090' // Comments
  const base08 = '#e06c75' // Error, deleted
  const base09 = '#f39c12' // Number, boolean
  const base0A = '#ffcb6b' // Keywords
  const base0B = '#98c379' // Strings
  const base0C = '#56b6c2' // Classes, types
  const base0D = '#61afef' // Functions, methods
  const base0E = '#c678dd' // Operators, brackets
  const base0F = '#be5046' // Special elements
  const invalid = '#e06c75' // Error highlighting

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base0A, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base0A, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base0A, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base0D },
    { tag: [tags.variableName], color: base0D },
    { tag: [tags.propertyName], color: base0C, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0C },
    { tag: [tags.className], color: base0C, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0D, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0E },
    { tag: [tags.bracket], color: base0E },
    { tag: [tags.brace], color: base0E },
    { tag: [tags.punctuation], color: base0E },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base0D },
    { tag: [tags.definition(tags.variableName)], color: base0D },

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
    { tag: [tags.special(tags.string), tags.regexp], color: base0B },
    { tag: tags.string, color: base0B },

    // Comments and documentation
    { tag: tags.meta, color: base08 },
    { tag: tags.comment, fontStyle: 'italic', color: base06 },
    { tag: tags.docComment, fontStyle: 'italic', color: base06 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0A },
    { tag: [tags.attributeName], color: base0D },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0A },
    { tag: [tags.strong], fontWeight: 'bold', color: base09 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0D },

    // Links and URLs
    { tag: [tags.link], color: base0F, fontWeight: '500', textDecoration: 'underline' },
    { tag: [tags.url], color: base0B, textDecoration: 'underline' },

    // Special states
    { tag: [tags.invalid], color: base01, textDecoration: 'underline wavy' },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Additional specific styles
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base0E },
    { tag: tags.angleBracket, color: base0E },
    { tag: tags.monospace, color: base01 },
    { tag: [tags.contentSeparator], color: base0D },
    { tag: tags.quote, color: base06 },
  ])

  return syntaxHighlighting(highlightStyle)
}
