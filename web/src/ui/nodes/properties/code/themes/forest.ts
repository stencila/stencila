import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Forest theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-forest
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/forest/src/index.ts
 */
export function forestTheme(): Extension {
  // Color palette
  const base01 = '#d0d8e0' // Foreground
  const base03 = '#607d8b' // Comments
  const base05 = '#a9d3ab' // Keywords
  const base06 = '#4db6ac' // Variables
  const base07 = '#78aadc' // Functions
  const base08 = '#ef5350' // Strings
  const base09 = '#bc8f6a' // Numbers
  const base0A = '#ffb74d' // Classes
  const base0B = '#9ccc65' // Properties
  const base0C = '#4dd0e1' // Special chars
  const base0D = '#7986cb' // Tags
  const base0E = '#ba68c8' // Operators
  const invalid = '#ff5252' // Error highlight

  const highlightStyle = HighlightStyle.define([
    // Keywords and language constructs
    { tag: tags.keyword, color: base05, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base05, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base05, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base06 },
    { tag: [tags.variableName], color: base06 },
    { tag: [tags.propertyName], color: base0B, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0A },
    { tag: [tags.className], color: base0A, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base06, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0E },
    { tag: [tags.bracket], color: base0E },
    { tag: [tags.brace], color: base0E },
    { tag: [tags.punctuation], color: base0E },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base07 },
    { tag: [tags.definition(tags.variableName)], color: base07 },

    // Constants and literals
    { tag: tags.number, color: base09 },
    { tag: tags.changed, color: base09 },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base09, fontStyle: 'italic' },
    { tag: tags.self, color: base09 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base09, fontWeight: 'bold' },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base09 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base08 },
    { tag: [tags.special(tags.string), tags.regexp], color: base08 },
    { tag: tags.string, color: base08 },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0A, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0D },
    { tag: [tags.attributeName], color: base0C },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base0A },
    { tag: [tags.strong], fontWeight: 'bold', color: base05 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base06 },

    // Links and URLs
    { tag: [tags.link], color: base0A, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: base0C, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base01, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base09 },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base0E },
  ])

  return syntaxHighlighting(highlightStyle)
}
