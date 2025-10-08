import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Android Studio theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-android-studio
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/android-studio/src/index.ts
 */
export function androidStudioTheme(): Extension {
  // Color palette
  const base05 = '#cc7832' // Keywords
  const base06 = '#6897bb' // Numbers, constants
  const base07 = '#9876aa' // Variables
  const base08 = '#787878' // Comments
  const base09 = '#bbb529' // Meta, annotations
  const base0A = '#6a8759' // Strings
  const base0B = '#ffc66d' // Class names, types
  const base0C = '#a9b7c6' // Attribute names
  const base0D = '#629755' // Function names, docs
  const base0E = '#d0d0ff' // Brackets
  const base0F = '#e8bf6a' // Tags
  const base10 = '#3c7abb' // Links
  const base11 = '#50a658' // URLs
  const invalid = '#ff5353' // Error highlighting
  const base01 = '#a9b7c6' // Foreground

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base05, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base05, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base05, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base07 },
    { tag: [tags.variableName], color: base07 },
    { tag: [tags.propertyName], color: base0A, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0B },
    { tag: [tags.className], color: base0B, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base07, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base05 },
    { tag: [tags.bracket], color: base0E },
    { tag: [tags.brace], color: base01 },
    { tag: [tags.punctuation], color: base01 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base01 },
    { tag: [tags.definition(tags.variableName)], color: base07 },

    // Constants and literals
    { tag: tags.number, color: base06 },
    { tag: tags.changed, color: base06 },
    { tag: tags.annotation, color: base09, fontStyle: 'italic' },
    { tag: tags.modifier, color: base09, fontStyle: 'italic' },
    { tag: tags.self, color: base05 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base06 },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base05 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0A },
    { tag: [tags.special(tags.string), tags.regexp], color: base0A },
    { tag: tags.string, color: base0A },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0B, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base08 },
    { tag: tags.comment, fontStyle: 'italic', color: base08 },
    { tag: tags.docComment, fontStyle: 'italic', color: base0D },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0F },
    { tag: [tags.attributeName], color: base0C },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0B },
    { tag: [tags.strong], fontWeight: 'bold' },
    { tag: [tags.emphasis], fontStyle: 'italic' },

    // Links and URLs
    { tag: [tags.link], color: base10, fontWeight: '500' },
    { tag: [tags.url], color: base11, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: invalid, textDecoration: 'underline wavy' },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base06 },
    { tag: tags.deleted, color: invalid },
    { tag: tags.labelName, color: base0D },
  ])

  return syntaxHighlighting(highlightStyle)
}
