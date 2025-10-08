import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * GitHub Light theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-github-light
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/github-light/src/index.ts
 */
export function githubLightTheme(): Extension {
  // Color palette
  const base01 = '#24292e' // Foreground
  const base05 = '#116329' // Tag names
  const base06 = '#6a737d' // Comments, brackets
  const base07 = '#6f42c1' // Classes, properties
  const base08 = '#005cc5' // Variables, attributes
  const base09 = '#d73a49' // Keywords, types
  const base0A = '#032f62' // Strings, regexps
  const base0B = '#22863a' // Names, quotes
  const base0C = '#e36209' // Atoms, booleans
  const invalid = '#cb2431' // Invalid color
  const linkColor = '#0969da' // Bright blue for links
  const visitedLinkColor = '#8250df' // Purple for visited links

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base09, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base09, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base09, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base08 },
    { tag: [tags.variableName], color: base08 },
    { tag: [tags.propertyName], color: base07, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base09 },
    { tag: [tags.className], color: base07, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base08, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base01 },
    { tag: [tags.bracket], color: base06 },
    { tag: [tags.brace], color: base06 },
    { tag: [tags.punctuation], color: base06 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base0B },
    { tag: [tags.definition(tags.variableName)], color: base08 },

    // Constants and literals
    { tag: tags.number, color: base0C },
    { tag: tags.changed, color: base0C },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base0C, fontStyle: 'italic' },
    { tag: tags.self, color: base0C },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0C },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base0C },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0B },
    { tag: [tags.special(tags.string), tags.regexp], color: base0A },
    { tag: tags.string, color: base0A },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base09, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base06 },
    { tag: tags.comment, fontStyle: 'italic', color: base06 },
    { tag: tags.docComment, fontStyle: 'italic', color: base06 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base05 },
    { tag: [tags.attributeName], color: base07 },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base08 },
    { tag: [tags.strong], fontWeight: 'bold', color: base08 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0A },

    // Links and URLs
    { tag: [tags.link], color: visitedLinkColor, fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: linkColor, textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base01, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0C },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base06 },
  ])

  return syntaxHighlighting(highlightStyle)
}
