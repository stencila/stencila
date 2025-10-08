import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Monokai theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-monokai
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/monokai/src/index.ts
 */
export function monokaiTheme(): Extension {
  // Color palette
  const base01 = '#f8f8f2' // Foreground
  const base02 = '#88846f' // Comments, invisibles
  const base04 = '#F92672' // Keyword, Storage, Tag - Pink
  const base05 = '#FD971F' // Variable, Parameter - Orange
  const base06 = '#66D9EF' // Function, Type - Blue
  const base07 = '#E6DB74' // String, RegExp - Yellow
  const base08 = '#AE81FF' // Constant, Number - Purple
  const base09 = '#A6E22E' // Class, Heading - Green
  const invalid = '#F44747' // Error color - Red

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base04, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base04, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base04, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base05 },
    { tag: [tags.variableName], color: base05 },
    { tag: [tags.propertyName], color: base09, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base06, fontStyle: 'italic' },
    { tag: [tags.className], color: base09, fontStyle: 'italic' },
    { tag: [tags.namespace], color: base05, fontStyle: 'italic' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base04 },
    { tag: [tags.bracket], color: base01 },
    { tag: [tags.brace], color: base01 },
    { tag: [tags.punctuation], color: base01 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName), tags.labelName], color: base06 },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base06 },
    { tag: [tags.definition(tags.variableName)], color: base05 },

    // Constants and literals
    { tag: tags.number, color: base08 },
    { tag: tags.changed, color: base08 },
    { tag: tags.annotation, color: invalid, fontStyle: 'italic' },
    { tag: tags.modifier, color: base08, fontStyle: 'italic' },
    { tag: tags.self, color: base08 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base08 },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base08 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base09 },
    { tag: [tags.special(tags.string), tags.regexp], color: base07 },
    { tag: tags.string, color: base07 },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base06, fontWeight: 'bold' },

    // Comments and documentation
    { tag: tags.meta, color: base02 },
    { tag: tags.comment, fontStyle: 'italic', color: base02 },
    { tag: tags.docComment, fontStyle: 'italic', color: base02 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base04 },
    { tag: [tags.attributeName], color: base09 },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base09 },
    { tag: [tags.strong], fontWeight: 'bold', color: base05 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base05 },

    // Links and URLs
    { tag: [tags.link], color: base08, fontWeight: '500', textDecoration: 'underline' },
    { tag: [tags.url], color: base06, textDecoration: 'underline' },

    // Special states
    { tag: [tags.invalid], color: base01, textDecoration: 'underline wavy' },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Additional specific styles
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base01 },
    { tag: tags.angleBracket, color: base01 },
    { tag: tags.monospace, color: base01 },
    { tag: [tags.contentSeparator], color: base05 },
    { tag: tags.quote, color: base02 },
  ])

  return syntaxHighlighting(highlightStyle)
}
