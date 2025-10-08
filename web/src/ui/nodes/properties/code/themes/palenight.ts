import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Palenight theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-palenight
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/palenight/src/index.ts
 */
export function palenightTheme(): Extension {
  // Color palette
  const base01 = '#A6ACCD' // Foreground
  const base03 = '#676E95' // Comments
  const base05 = '#C3E88D' // Strings
  const base06 = '#82AAFF' // Keywords, Functions
  const base07 = '#C792EA' // Classes, Types
  const base08 = '#F78C6C' // Numbers, Constants
  const base09 = '#FFCB6B' // Classes, Attributes
  const base0A = '#89DDFF' // Punctuation, Operators
  const base0B = '#FF5370' // Tags, Errors
  const base0C = '#BB80B3' // Special elements
  const base0D = '#80CBC4' // Properties
  const invalid = '#FF5370'
  const linkColor = base06
  const visitedLinkColor = base07

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base06, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base06, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base06, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base01 },
    { tag: [tags.variableName], color: base01 },
    { tag: [tags.propertyName], color: base0D, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base07 },
    { tag: [tags.className], color: base09, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base09, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0A },
    { tag: [tags.bracket], color: base0A },
    { tag: [tags.brace], color: base0A },
    { tag: [tags.punctuation], color: base0A },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base06 },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base06 },
    { tag: [tags.definition(tags.variableName)], color: base08 },

    // Constants and literals
    { tag: tags.number, color: base08 },
    { tag: tags.changed, color: base08 },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0C, fontStyle: 'italic' },
    { tag: tags.self, color: base07 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base08 },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base0C },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base05 },
    { tag: [tags.special(tags.string), tags.regexp], color: base05 },
    { tag: tags.string, color: base05 },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base07, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base0A },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base0B },
    { tag: [tags.attributeName], color: base09 },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base09 },
    { tag: [tags.strong], fontWeight: 'bold', color: base09 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base05 },

    // Links and URLs
    { tag: [tags.link], color: visitedLinkColor, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: linkColor, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base01, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base08 },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base0A },
  ])

  return syntaxHighlighting(highlightStyle)
}
