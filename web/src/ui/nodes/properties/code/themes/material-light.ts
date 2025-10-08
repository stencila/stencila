import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Material Light theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-material-light
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/material-light/src/index.ts
 */
export function materialLightTheme(): Extension {
  // Color palette
  const base03 = '#757575' // Comments
  const base05 = '#424242' // Default foreground
  const base09 = '#ff3e00' // Functions
  const base0A = '#FF00E9FF' // Regex, special
  const base0C = '#ff9800' // Types
  const base0D = '#00acc1' // Keywords
  const base0E = '#3949ab' // Operators
  const base0F = '#8e24aa' // Atoms, bools
  const base10 = '#43a047' // Strings
  const base11 = '#00897b' // Properties
  const base12 = '#1e88e5' // Headings, labels
  const invalid = '#f44336'
  const linkColor = base0C
  const visitedLinkColor = base0F

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base0D, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base0D, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base0D, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base05 },
    { tag: [tags.variableName], color: base05 },
    { tag: [tags.propertyName], color: base11, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0C },
    { tag: [tags.className], color: base0C, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base0E, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base0E },
    { tag: [tags.bracket], color: base0F },
    { tag: [tags.brace], color: base0F },
    { tag: [tags.punctuation], color: base03 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName)], color: base09 },
    { tag: [tags.labelName], color: base12, fontStyle: 'italic' },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base09 },
    { tag: [tags.definition(tags.variableName)], color: base0A },

    // Constants and literals
    { tag: tags.number, color: base0C },
    { tag: tags.changed, color: base0C },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0C, fontStyle: 'italic' },
    { tag: tags.self, color: base0C },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0C },
    { tag: [tags.atom, tags.bool], color: base0F },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base10 },
    { tag: tags.string, color: base10 },
    { tag: [tags.special(tags.string), tags.regexp], color: base0A },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0A, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base0A },

    // Comments and documentation
    { tag: tags.meta, color: base03 },
    { tag: tags.comment, fontStyle: 'italic', color: base03 },
    { tag: tags.docComment, fontStyle: 'italic', color: base03 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base09 },
    { tag: [tags.attributeName], color: base05 },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base11 },
    { tag: tags.heading1, color: base12 },
    { tag: tags.heading2, color: base0C },
    { tag: tags.heading3, color: base0D },
    { tag: tags.heading4, color: base0E },
    { tag: tags.heading5, color: base0F },
    { tag: tags.heading6, color: base10 },
    { tag: [tags.strong], fontWeight: 'bold', color: base0E },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0C },

    // Links and URLs
    { tag: [tags.link], color: visitedLinkColor, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: linkColor, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base05, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0C },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base0F },
  ])

  return syntaxHighlighting(highlightStyle)
}
