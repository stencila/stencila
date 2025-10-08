import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Gruvbox Dark theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-gruvbox-dark
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/gruvbox-dark/src/index.ts
 */
export function gruvboxDarkTheme(): Extension {
  // Color palette
  const base05 = '#928374' // Comments
  const base07 = '#ebdbb2' // Light foreground
  const base08 = '#fb4934' // Keywords
  const base09 = '#b8bb26' // Strings
  const base0A = '#fabd2f' // Functions
  const base0B = '#83a598' // Variables
  const base0C = '#d3869b' // Numbers
  const base0D = '#8ec07c' // Types
  const base0E = '#fe8019' // Constants
  const invalid = base08
  const linkColor = base0B
  const visitedLinkColor = base0C

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base08, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base08, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base08, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base0B },
    { tag: [tags.variableName], color: base0B },
    { tag: [tags.propertyName], color: base0D, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0D },
    { tag: [tags.className], color: base0A, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0B, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base07 },
    { tag: [tags.bracket], color: base05 },
    { tag: [tags.brace], color: base05 },
    { tag: [tags.punctuation], color: base05 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base0A },
    { tag: [tags.definition(tags.variableName)], color: base0B },

    // Constants and literals
    { tag: tags.number, color: base0C },
    { tag: tags.changed, color: base0C },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0C, fontStyle: 'italic' },
    { tag: tags.self, color: base0C },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0E },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base0E },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base09 },
    { tag: [tags.special(tags.string), tags.regexp], color: base09 },
    { tag: tags.string, color: base09 },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0D, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base05 },
    { tag: tags.comment, fontStyle: 'italic', color: base05 },
    { tag: tags.docComment, fontStyle: 'italic', color: base05 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base08 },
    { tag: [tags.attributeName], color: base0A },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0A },
    { tag: [tags.strong], fontWeight: 'bold', color: base0A },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base09 },

    // Links and URLs
    { tag: [tags.link], color: visitedLinkColor, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: linkColor, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base07, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0E },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base05 },
  ])

  return syntaxHighlighting(highlightStyle)
}
