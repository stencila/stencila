import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * VS Code Dark theme for CodeMirror
 *
 * Based on @fsegurai/codemirror-theme-vscode-dark
 * Copyright (c) fsegurai
 * Licensed under MIT License
 * Source: https://github.com/fsegurai/codemirror-themes/blob/main/packages/vscode-dark/src/index.ts
 */
export function vsCodeDarkTheme(): Extension {
  // Color palette
  const base05 = '#d4d4d4' // Default foreground
  const base08 = '#569cd6' // Keywords
  const base09 = '#c586c0' // Control keywords
  const base0A = '#9cdcfe' // Variables
  const base0B = '#4ec9b0' // Classes, types
  const base0C = '#dcdcaa' // Functions
  const base0D = '#b5cea8' // Numbers
  const base0E = '#ce9178' // Strings
  const base10 = '#d7ba7d' // Modified elements
  const base11 = '#6a9955' // Comments
  const invalid = '#f44747'

  const highlightStyle = HighlightStyle.define([
    // Keywords and control flow
    { tag: tags.keyword, color: base08, fontWeight: 'bold' },
    { tag: tags.controlKeyword, color: base09, fontWeight: 'bold' },
    { tag: tags.moduleKeyword, color: base08, fontWeight: 'bold' },

    // Names and variables
    { tag: [tags.name, tags.deleted, tags.character, tags.macroName], color: base05 },
    { tag: [tags.variableName], color: base0A },
    { tag: [tags.propertyName], color: base0A, fontStyle: 'normal' },

    // Classes and types
    { tag: [tags.typeName], color: base0B },
    { tag: [tags.className], color: base0B, fontStyle: 'normal' },
    { tag: [tags.namespace], color: base05, fontStyle: 'normal' },

    // Operators and punctuation
    { tag: [tags.operator, tags.operatorKeyword], color: base05 },
    { tag: [tags.bracket], color: base05 },
    { tag: [tags.brace], color: base05 },
    { tag: [tags.punctuation], color: base05 },

    // Functions and parameters
    { tag: [tags.function(tags.variableName)], color: base0C },
    { tag: [tags.labelName], color: base0C, fontStyle: 'normal' },
    { tag: [tags.definition(tags.function(tags.variableName))], color: base0C },
    { tag: [tags.definition(tags.variableName)], color: base0A },

    // Constants and literals
    { tag: tags.number, color: base0D },
    { tag: tags.changed, color: base10 },
    { tag: tags.annotation, color: base10, fontStyle: 'italic' },
    { tag: tags.modifier, color: base08, fontStyle: 'normal' },
    { tag: tags.self, color: base08 },
    { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: base0A },
    { tag: [tags.atom, tags.bool, tags.special(tags.variableName)], color: base08 },

    // Strings and regex
    { tag: [tags.processingInstruction, tags.inserted], color: base0E },
    { tag: [tags.special(tags.string), tags.regexp], color: '#d16969' },
    { tag: tags.string, color: base0E },

    // Punctuation and structure
    { tag: tags.definition(tags.typeName), color: base0B, fontWeight: 'bold' },
    { tag: [tags.definition(tags.name), tags.separator], color: base05 },

    // Comments and documentation
    { tag: tags.meta, color: base11 },
    { tag: tags.comment, fontStyle: 'italic', color: base11 },
    { tag: tags.docComment, fontStyle: 'italic', color: base11 },

    // HTML/XML elements
    { tag: [tags.tagName], color: base08 },
    { tag: [tags.attributeName], color: base0A },

    // Markdown and text formatting
    { tag: [tags.heading], fontWeight: 'bold', color: base08 },
    { tag: tags.heading1, color: base08, fontWeight: 'bold' },
    { tag: tags.heading2, color: base08 },
    { tag: tags.heading3, color: base08 },
    { tag: tags.heading4, color: base08 },
    { tag: tags.heading5, color: base08 },
    { tag: tags.heading6, color: base08 },
    { tag: [tags.strong], fontWeight: 'bold', color: base08 },
    { tag: [tags.emphasis], fontStyle: 'italic', color: base0B },

    // Links and URLs
    { tag: [tags.link], color: '#3794ff', fontWeight: '500', textDecoration: 'underline', textUnderlinePosition: 'under' },
    { tag: [tags.url], color: '#3794ff', textDecoration: 'underline', textUnderlineOffset: '2px' },

    // Special states
    { tag: [tags.invalid], color: base05, textDecoration: 'underline wavy', borderBottom: `1px wavy ${invalid}` },
    { tag: [tags.strikethrough], color: invalid, textDecoration: 'line-through' },

    // Enhanced syntax highlighting
    { tag: tags.constant(tags.name), color: base0A },
    { tag: tags.deleted, color: invalid },
    { tag: tags.squareBracket, color: base05 },
  ])

  return syntaxHighlighting(highlightStyle)
}
