import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Andromeda theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-andromeda
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/andromeda/src/index.ts
 */
export function andromedaTheme(): Extension {
  // Color palette
  const base05 = '#d667ff' // Keyword, Storage
  const base06 = '#24e3c3' // Variable, Parameter
  const base07 = '#ffdd80' // Function, Type, Class
  const base08 = '#a6e07a' // String, RegExp
  const base09 = '#ff7057' // Constant, Number
  const base0A = '#a8aab9' // Comment
  const base0B = '#ff40b3' // Heading
  const base0C = '#fd3681' // Tag
  const base0D = '#c7c7ff' // Brackets/punctuation
  const base0E = '#6ae4b9' // Special elements
  const base0F = '#3c94ff' // Attributes and links
  const invalid = '#ff3162' // Invalid

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base05, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base05, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base05, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base06 },
    { tag: [tags.variableName], color: base06 },
    { tag: [tags.propertyName], color: base06, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base07 },
    { tag: [tags.className], color: base07, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0E, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0D },
    { tag: [tags.bracket], color: base0D },
    { tag: [tags.brace], color: base0D },
    { tag: [tags.punctuation], color: base0D },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base07 },
    { tag: [tags.definition(tags.variableName)], color: base06 },

    // Constants and literals
    { tag: tags.number, color: base09 },
    { tag: tags.changed, color: base09 },
    { tag: tags.annotation, color: base0F, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0F, fontStyle: 'italic' },
    { tag: tags.self, color: base09 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base09 },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base09 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base08 },
    { tag: [tags.special(tags.string), tags.regexp], color: base08 },
    { tag: tags.string, color: base08 },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base07, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base0A },
    { tag: tags.comment, fontStyle: 'italic', color: base0A },
    { tag: tags.docComment, fontStyle: 'italic', color: base0A },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0C },
    { tag: [tags.attributeName], color: base0F },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0B },
    { tag: [tags.strong], fontWeight: 'bold', color: base09 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0E },

    // Links and URLs
    { tag: [tags.link], color: base0F, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: base0E, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: invalid, textDecoration: 'underline wavy' },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base09 },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base0D },
  ])

  return syntaxHighlighting(highlightStyle)
}
