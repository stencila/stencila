import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { tags as t } from '@lezer/highlight'

/**
 * Create a dynamic CodeMirror theme that reads from Stencila CSS variables.
 * This allows the editor to automatically adapt to theme changes.
 */
export function createTheme(): Extension {
  // Base editor theme - layout, colors, and UI elements
  const editorTheme = EditorView.theme({
    '&': {
      color: 'var(--code-color)',
      backgroundColor: 'var(--code-background)',
      fontFamily: 'var(--code-font-family)',
      fontSize: 'var(--code-font-size-block)',
      lineHeight: 'var(--code-line-height)',
      border: 'var(--code-border-width) solid var(--code-border-color)',
    },
    '&.cm-focused': {
      backgroundColor: 'var(--code-focused-background)',
    },
    '.cm-content': {
      padding: 'var(--code-padding-block) var(--code-padding-inline)',
      caretColor: 'var(--code-caret-color)',
    },
    '.cm-scroller': {
      overflowX: 'auto',
    },
    '.cm-focused .cm-cursor': {
      borderLeftColor: 'var(--code-caret-color)',
    },
    '.cm-selectionBackground, ::selection': {
      backgroundColor: 'var(--code-selection-background)',
    },
    '.cm-selectionMatch': {
      backgroundColor: 'var(--code-selection-match-background)',
    },
    '.cm-activeLine': {
      backgroundColor: 'var(--code-cursor-line-background)',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--code-gutter-background)',
      color: 'var(--code-gutter-color)',
      border: 'none',
      borderRight: 'var(--code-gutter-border-width) solid var(--code-gutter-border-color)',
      paddingLeft: 'var(--code-gutter-padding)',
      paddingRight: 'var(--code-gutter-padding)',
      minWidth: 'var(--code-gutter-min-width)',
    },
    '.cm-gutterElement': {
      paddingLeft: '0',
      paddingRight: '0',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'var(--code-gutter-active-line-background)',
      color: 'var(--code-gutter-active-line-color)',
    },
    '.cm-lineNumbers .cm-gutterElement': {
      color: 'var(--code-gutter-color)',
      fontSize: 'var(--code-font-size-block)',
      fontFamily: 'var(--code-font-family)',
    },
    // Diagnostic styles (for error messages, tooltips, etc.)
    '.cm-diagnostic': {
      paddingLeft: '16px',
      paddingRight: '16px',
      borderBottom: '1px solid var(--border-color-muted)',
    },
    '.cm-diagnostic:last-child': {
      borderBottom: '0px',
    },
    // Tooltip styles
    '.cm-tooltip:has(> .cm-provenance-tooltip)': {
      minWidth: '30px',
      border: 'none',
      color: '#ffffff',
      backgroundColor: 'var(--sl-tooltip-background-color)',
      fontFamily: 'var(--sl-tooltip-font-family)',
      borderRadius: 'var(--sl-tooltip-border-radius)',
      fontSize: 'var(--sl-tooltip-font-size)',
      fontWeight: 'var(--sl-tooltip-font-weight)',
      lineHeight: 'var(--sl-tooltip-line-height)',
      padding: 'var(--sl-tooltip-padding)',
    },
    'div.cm-tooltip-arrow::after': {
      borderBottomColor: `var(--sl-tooltip-background-color) !important`,
    },
    'div.cm-tooltip-arrow::before': {
      borderBottomColor: `var(--sl-tooltip-background-color) !important`,
    },
  })

  // Syntax highlighting theme - maps @lezer/highlight tags to CSS variables
  const syntaxHighlightStyle = HighlightStyle.define([
    // Comments
    { tag: t.comment, color: 'var(--syntax-comment)' },
    { tag: t.blockComment, color: 'var(--syntax-comment-block)' },
    { tag: t.lineComment, color: 'var(--syntax-comment-line)' },
    { tag: t.docComment, color: 'var(--syntax-comment-doc)' },

    // Keywords
    { tag: t.keyword, color: 'var(--syntax-keyword)' },
    { tag: t.controlKeyword, color: 'var(--syntax-keyword-control)' },
    { tag: t.operatorKeyword, color: 'var(--syntax-keyword-operator)' },
    { tag: t.modifier, color: 'var(--syntax-keyword-modifier)' },
    { tag: t.definitionKeyword, color: 'var(--syntax-keyword-definition)' },

    // Strings and literals
    { tag: t.string, color: 'var(--syntax-string)' },
    { tag: t.special(t.string), color: 'var(--syntax-string-special)' },
    { tag: t.regexp, color: 'var(--syntax-string-regex)' },
    { tag: t.escape, color: 'var(--syntax-string-escape)' },

    // Numbers and values
    { tag: t.number, color: 'var(--syntax-number)' },
    { tag: t.integer, color: 'var(--syntax-number-integer)' },
    { tag: t.float, color: 'var(--syntax-number-float)' },
    { tag: t.bool, color: 'var(--syntax-boolean)' },
    { tag: t.null, color: 'var(--syntax-constant)' },

    // Constants and built-ins
    { tag: t.constant(t.name), color: 'var(--syntax-constant)' },
    { tag: t.standard(t.name), color: 'var(--syntax-constant-builtin)' },

    // Variables
    { tag: t.variableName, color: 'var(--syntax-variable)' },
    { tag: t.standard(t.variableName), color: 'var(--syntax-variable-builtin)' },
    { tag: t.special(t.variableName), color: 'var(--syntax-variable-special)' },

    // Functions and methods
    { tag: t.function(t.variableName), color: 'var(--syntax-function)' },
    { tag: t.function(t.propertyName), color: 'var(--syntax-function-method)' },
    { tag: t.standard(t.function(t.variableName)), color: 'var(--syntax-function-builtin)' },

    // Classes and types
    { tag: t.className, color: 'var(--syntax-class)' },
    { tag: t.standard(t.className), color: 'var(--syntax-class-builtin)' },
    { tag: t.typeName, color: 'var(--syntax-type)' },
    { tag: t.standard(t.typeName), color: 'var(--syntax-type-builtin)' },

    // Namespaces and modules
    { tag: t.namespace, color: 'var(--syntax-namespace)' },
    { tag: t.macroName, color: 'var(--syntax-namespace)' },

    // Properties and attributes
    { tag: t.propertyName, color: 'var(--syntax-property)' },
    { tag: t.attributeName, color: 'var(--syntax-attribute)' },

    // Tags (for markup languages)
    { tag: t.tagName, color: 'var(--syntax-tag)' },
    { tag: t.angleBracket, color: 'var(--syntax-tag-angle)' },

    // Operators and punctuation
    { tag: t.operator, color: 'var(--syntax-operator)' },
    { tag: t.punctuation, color: 'var(--syntax-punctuation)' },
    { tag: t.bracket, color: 'var(--syntax-punctuation-bracket)' },
    { tag: t.separator, color: 'var(--syntax-punctuation-delimiter)' },

    // Meta and special
    { tag: t.meta, color: 'var(--syntax-meta)' },
    { tag: t.processingInstruction, color: 'var(--syntax-meta)' },

    // Invalid and deprecated
    { tag: t.invalid, color: 'var(--syntax-invalid)' },
    { tag: t.deleted, color: 'var(--syntax-invalid-deprecated)' },

    // Text emphasis (for markdown, etc.)
    { tag: t.emphasis, fontStyle: 'italic' },
    { tag: t.strong, fontWeight: 'bold' },
    { tag: t.strikethrough, textDecoration: 'line-through' },
  ])

  return [editorTheme, syntaxHighlighting(syntaxHighlightStyle)]
}
