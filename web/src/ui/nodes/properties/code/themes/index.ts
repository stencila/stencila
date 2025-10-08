import { Extension } from '@codemirror/state'
import { EditorView } from '@codemirror/view'

import { basicDarkTheme } from './basic-dark'
import { basicLightTheme } from './basic-light'
import { customTheme } from './custom'

/**
 * Get the value of a CSS variable from the document root
 */
function getCSSVariable(name: string): string {
  return getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim()
    .replace(/['"]/g, '') // Remove quotes if present
}

/**
 * Create the base editor theme for layout and UI elements.
 * This provides Stencila-specific styling like borders, padding, gutters.
 */
function createEditorTheme(): Extension {
  return EditorView.theme({
    '&': {
      backgroundColor: 'var(--code-background)',
      border: 'var(--code-border-width) solid var(--code-border-color)',
    },
    '&.cm-focused': {
      backgroundColor: 'var(--code-focused-background)',
    },
    '.cm-content': {
      padding: 'var(--code-padding-block)',
      color: 'var(--code-color)',
      fontFamily: 'var(--code-font-family)',
      fontSize: 'var(--code-font-size-block)',
      lineHeight: 'var(--code-line-height)',
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
      borderRight:
        'var(--code-gutter-border-width) solid var(--code-gutter-border-color)',
      paddingLeft: 'var(--code-gutter-padding)',
      paddingRight: 'var(--code-gutter-padding)',
      minWidth: 'var(--code-gutter-min-width)',
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
}

/**
 * Create a dynamic CodeMirror theme that either uses a named theme
 * or reads from Stencila CSS variables when set to 'custom'.
 */
export function createTheme(): Extension {
  const themeName = getCSSVariable('--code-theme') || 'basic-light'

  // Editor theme is always included
  const editorTheme = createEditorTheme()

  // Select syntax highlighting theme
  const syntaxTheme = (() => {
    switch (themeName) {
      case 'custom':
        return customTheme()
      case 'basic-dark':
        return basicDarkTheme()
      case 'basic-light':
      default:
        return basicLightTheme()
    }
  })()

  return [editorTheme, syntaxTheme]
}
