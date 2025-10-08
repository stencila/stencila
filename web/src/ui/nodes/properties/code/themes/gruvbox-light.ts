import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Gruvbox Light theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-gruvbox-light
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/gruvbox-light/src/index.ts
 */
export function gruvboxLightTheme(): Extension {
  // Color palette
  const base00 = '#3c3836' // Main foreground
  const base04 = '#928374' // Comments
  const base0A = '#9d0006' // Keywords
  const base0B = '#79740e' // Strings
  const base0C = '#b57614' // Functions
  const base0D = '#076678' // Variables
  const base0E = '#8f3f71' // Numbers
  const base0F = '#427b58' // Types
  const base10 = '#af3a03' // Constants
  const invalid = base0A
  const linkColor = base0D
  const visitedLinkColor = base0E

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base0A, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base0A, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base0A, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base0D },
    { tag: [tags.variableName], color: base0D },
    { tag: [tags.propertyName], color: base0F, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0F },
    { tag: [tags.className], color: base0C, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0D, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base00 },
    { tag: [tags.bracket], color: base04 },
    { tag: [tags.brace], color: base04 },
    { tag: [tags.punctuation], color: base04 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base0C },
    { tag: [tags.definition(tags.variableName)], color: base0D },

    // Constants and literals
    { tag: tags.number, color: base0E },
    { tag: tags.changed, color: base0E },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0E, fontStyle: 'italic' },
    { tag: tags.self, color: base0E },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base10 },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base10 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0B },
    { tag: [tags.special(tags.string), tags.regexp], color: base0B },
    { tag: tags.string, color: base0B },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0F, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base04 },
    { tag: tags.comment, fontStyle: 'italic', color: base04 },
    { tag: tags.docComment, fontStyle: 'italic', color: base04 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0A },
    { tag: [tags.attributeName], color: base0C },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0C },
    { tag: [tags.strong], fontWeight: 'bold', color: base0C },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0B },

    // Links and URLs
    { tag: [tags.link], color: visitedLinkColor, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: linkColor, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base00, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base10 },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base04 },
  ])

  return syntaxHighlighting(highlightStyle)
}
