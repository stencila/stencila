import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Nord theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-nord
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/nord/src/index.ts
 */
export function nordTheme(): Extension {
  // Color palette
  const base03 = '#4c566a' // Comments
  const base04 = '#d8dee9' // Foreground
  const base06 = '#8fbcbb' // Function names
  const base07 = '#88c0d0' // Classes, attributes
  const base08 = '#81a1c1' // Methods
  const base09 = '#5e81ac' // Keywords
  const base0A = '#bf616a' // Errors, brackets
  const base0B = '#d08770' // Numbers, constants
  const base0C = '#ebcb8b' // Types, classes
  const base0D = '#a3be8c' // Strings
  const base0E = '#b48ead' // Operators, special
  const linkColor = base08
  const visitedLinkColor = base0E

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base09, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base09, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base09, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base07 },
    { tag: [tags.variableName], color: base04 },
    { tag: [tags.propertyName], color: base07, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0C },
    { tag: [tags.className], color: base0C, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base08, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0E },
    { tag: [tags.bracket], color: base04 },
    { tag: [tags.brace], color: base06 },
    { tag: [tags.punctuation], color: base04 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base06 },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base06 },
    { tag: [tags.definition(tags.variableName)], color: base0B },

    // Constants and literals
    { tag: tags.number, color: base0B },
    { tag: tags.changed, color: base0E },
    { tag: tags.annotation, color: base0A, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0E, fontStyle: 'italic' },
    { tag: tags.self, color: base0E },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0B },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base0E },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base06 },
    { tag: [tags.special(tags.string), tags.regexp], color: base0E },
    { tag: tags.string, color: base0D },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0C, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base0D },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0A },
    { tag: [tags.attributeName], color: base0C },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base08 },
    { tag: [tags.strong], fontWeight: 'bold', color: base07 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0C },

    // Links and URLs
    { tag: [tags.link], color: visitedLinkColor, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: linkColor, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base04, textDecoration: 'underline wavy', borderBottom: `1px wavy ${base0A}` },
    { tag: [tags.strikethrough], color: base0A, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0B },
    { tag: tags.deleted, color: base0A },
    { tag: tags.squareBracket, color: base04 },
  ])

  return syntaxHighlighting(highlightStyle)
}
