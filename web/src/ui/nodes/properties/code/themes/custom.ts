import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags as t } from '@lezer/highlight'

/**
 * Custom theme for CodeMirror
 *
 * This uses CSS variables for full customization.
 * Set --code-theme: 'custom' to use this theme and customize
 * all syntax highlighting colors via CSS variables.
 */
export function customTheme(): Extension {
  // Syntax highlighting theme - maps @lezer/highlight tags to CSS variables
  const syntaxHighlightStyle = HighlightStyle.define([
    // Comments
    { tag: t.comment, color: 'var(--code-comment)' },
    { tag: t.blockComment, color: 'var(--code-comment-block)' },
    { tag: t.lineComment, color: 'var(--code-comment-line)' },
    { tag: t.docComment, color: 'var(--code-comment-doc)' },

    // Keywords
    { tag: t.keyword, color: 'var(--code-keyword)' },
    { tag: t.controlKeyword, color: 'var(--code-keyword-control)' },
    { tag: t.operatorKeyword, color: 'var(--code-keyword-operator)' },
    { tag: t.modifier, color: 'var(--code-keyword-modifier)' },
    { tag: t.definitionKeyword, color: 'var(--code-keyword-definition)' },

    // Strings and literals
    { tag: t.string, color: 'var(--code-string)' },
    { tag: t.special(t.string), color: 'var(--code-string-special)' },
    { tag: t.regexp, color: 'var(--code-string-regex)' },
    { tag: t.escape, color: 'var(--code-string-escape)' },

    // Numbers and values
    { tag: t.number, color: 'var(--code-number)' },
    { tag: t.integer, color: 'var(--code-number-integer)' },
    { tag: t.float, color: 'var(--code-number-float)' },
    { tag: t.bool, color: 'var(--code-boolean)' },
    { tag: t.null, color: 'var(--code-constant)' },

    // Constants and built-ins
    { tag: t.constant(t.name), color: 'var(--code-constant)' },
    { tag: t.standard(t.name), color: 'var(--code-constant-builtin)' },

    // Variables
    { tag: t.variableName, color: 'var(--code-variable)' },
    { tag: t.standard(t.variableName), color: 'var(--code-variable-builtin)' },
    { tag: t.special(t.variableName), color: 'var(--code-variable-special)' },

    // Functions and methods
    { tag: t.function(t.variableName), color: 'var(--code-function)' },
    { tag: t.function(t.propertyName), color: 'var(--code-function-method)' },
    { tag: t.standard(t.function(t.variableName)), color: 'var(--code-function-builtin)' },

    // Classes and types
    { tag: t.className, color: 'var(--code-class)' },
    { tag: t.standard(t.className), color: 'var(--code-class-builtin)' },
    { tag: t.typeName, color: 'var(--code-type)' },
    { tag: t.standard(t.typeName), color: 'var(--code-type-builtin)' },

    // Namespaces and modules
    { tag: t.namespace, color: 'var(--code-namespace)' },
    { tag: t.macroName, color: 'var(--code-namespace)' },

    // Properties and attributes
    { tag: t.propertyName, color: 'var(--code-property)' },
    { tag: t.attributeName, color: 'var(--code-attribute)' },

    // Tags (for markup languages)
    { tag: t.tagName, color: 'var(--code-tag)' },
    { tag: t.angleBracket, color: 'var(--code-tag-angle)' },

    // Operators and punctuation
    { tag: t.operator, color: 'var(--code-operator)' },
    { tag: t.punctuation, color: 'var(--code-punctuation)' },
    { tag: t.bracket, color: 'var(--code-punctuation-bracket)' },
    { tag: t.separator, color: 'var(--code-punctuation-delimiter)' },

    // Meta and special
    { tag: t.meta, color: 'var(--code-meta)' },
    { tag: t.processingInstruction, color: 'var(--code-meta)' },

    // Invalid and deprecated
    { tag: t.invalid, color: 'var(--code-invalid)' },
    { tag: t.deleted, color: 'var(--code-invalid-deprecated)' },

    // Text emphasis (for markdown, etc.)
    { tag: t.emphasis, fontStyle: 'italic' },
    { tag: t.strong, fontWeight: 'bold' },
    { tag: t.strikethrough, textDecoration: 'line-through' },
  ])

  return syntaxHighlighting(syntaxHighlightStyle)
}
