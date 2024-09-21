import { Diagnostic } from '@codemirror/lint'
import { EditorView, Decoration, hoverTooltip } from '@codemirror/view'

import { CompilationMessage } from '../../../../nodes/compilation-message'
import { ExecutionMessage } from '../../../../nodes/execution-message'
import {
  ProvenanceOpacityLevel,
  getProvenanceOpacity,
} from '../../icons-and-colours'
import { getTooltipContent } from '../authorship/utils'

import type { AuthorshipMarker } from './types'

/**
 * Custom CSS for CodeMirror editor
 */
export const stencilaTheme = EditorView.theme({
  '.cm-content,.cm-lineNumbers': {
    fontFamily: "'IBM Plex Mono', Menlo, Courier, mono",
    fontSize: '0.85rem'
  },
  '.cm-diagnostic': {
    fontFamily: 'Inter, mono',
    paddingLeft: '16px',
    paddingRight: '16px',
    borderBottom: '1px solid #dedede', // grey-200
  },
  '.cm-diagnostic:last-child': {
    borderBottom: '0px',
  },
  '.cm-tooltip:has(> .cm-provenance-tooltip)': {
    minWidth: '30px',
    border: 'none',
    color: '#ffffff',
    // use sl tooltip variables for consistency
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

/**
 * Creates a set of CodeMirror mark type decorations from the
 * array of `AuthorshipMarkers`
 */
export const createLinterDiagnostics = (
  view: EditorView,
  messages: (CompilationMessage | ExecutionMessage)[]
): Diagnostic[] => {
  const doc = view.state.doc
  return messages.map((msg): Diagnostic => {
    let from = 0
    let to = doc.length
    if (msg.codeLocation) {
      const [startLine, startCol, endLine, endCol] = msg.codeLocation
      if (startLine >= 0) {
        const startLineInfo = doc.line(startLine + 1)
        from =
          startCol >= 0 ? startLineInfo.from + startCol : startLineInfo.from
        if (endLine >= 0) {
          const endLineInfo = doc.line(endLine + 1)
          to = endCol >= 0 ? endLineInfo.from + endCol : endLineInfo.to
        } else {
          to = startLineInfo.to
        }
      }
    }

    const level = msg.level
    const severity =
      level === 'Error' || level === 'Exception'
        ? 'error'
        : level === 'Warning'
          ? 'warning'
          : 'info'

    const message = `${msg.errorType ?? level}: ${msg.message}`

    return {
      from,
      to,
      severity,
      message,
    }
  })
}

/**
 * Creates a set of CodeMirror mark type decorations from the
 * array of `AuthorshipMarkers`
 *
 * @param marks AuthorshipMarker[]
 * @returns DecorationSet
 */
export const createProvenanceDecorations = (marks: AuthorshipMarker[]) =>
  Decoration.set(
    marks.map((mark) => {
      return Decoration.mark({
        tagName: 'span',
        class: `cm-authorship prov-lvl-${mark.mi}`,
        attributes: {
          style:
            mark.mi >= 0 && mark.mi <= 5
              ? `opacity: ${getProvenanceOpacity(mark.mi as ProvenanceOpacityLevel)};`
              : '',
        },
      }).range(mark.from, mark.to)
    })
  )

/**
 * Create a hover tooltip to display the authorship provenance information
 */
export const provenanceTooltip = (
  marks: AuthorshipMarker[],
  diagnostics: Diagnostic[]
) =>
  hoverTooltip((_, pos) => {
    // Disable tooltip if diagnostics are present, to avoid a merged tooltip
    if (diagnostics.length > 0) {
      return null
    }

    for (const mark of marks) {
      if (pos >= mark.from && pos <= mark.to) {
        return {
          pos,
          above: false,
          arrow: true,
          create: () => {
            const dom = document.createElement('div')
            dom.className = 'cm-provenance-tooltip'

            dom.textContent = getTooltipContent(mark.count, mark.provenance)

            return { dom, offset: { x: 0, y: 10 } }
          },
        }
      }
    }

    return null
  })
