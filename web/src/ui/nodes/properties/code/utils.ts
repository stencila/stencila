import { linter, Diagnostic } from '@codemirror/lint'
import { Extension } from '@codemirror/state'
import { EditorView, Decoration, hoverTooltip } from '@codemirror/view'
import { MessageLevel } from '@stencila/types'

import {
  ProvenanceHighlightLevel,
  getProvenanceHighlight,
} from '../../icons-and-colours'
import { ExecutionMessage } from '../execution-message'

import type { ProvenanceMarker } from './types'

/**
 * Convert the `ExecutionMessage.level` value into a @codemirror/lint `Severity` string
 * for the linting extension
 *
 * @param lvl `MessageLevel`
 * @returns 'Severity'
 */
const getMessageSeverity = (lvl: MessageLevel) =>
  lvl === 'Error' || lvl === 'Exception'
    ? 'error'
    : lvl === 'Warning'
      ? 'warning'
      : 'info'

/**
 * Returns a `linter` extension for a codemirror editor.
 * Creates a `Diagnostic` for each of the execution messages.
 * @param messages Array of `ExecutionMessage` objects
 * @returns codemirror linter `Extension`
 */
const executionMessageLinter = (messages: ExecutionMessage[]): Extension =>
  linter((view: EditorView) => {
    const diagnostics: Diagnostic[] = []
    const from = 0
    const to = view.state.doc.length

    messages.forEach((msg) => {
      const severity = getMessageSeverity(msg.level)
      diagnostics.push({
        from,
        to,
        severity,
        message: msg.message,
      })
    })

    return diagnostics
  })

/**
 * Custom css for the stencila extensions
 */
const stencilaTheme = EditorView.theme({
  '.cm-diagnostic': {
    fontFamily: 'mono',
    paddingLeft: '16px',
    paddingRight: '16px',
    borderBottom: '1px solid #dedede', // grey-200
  },
  '.cm-diagnostic:last-child': {
    borderBottom: '0px',
  },
  '.cm-tooltip': {
    minWidth: '30px',
    border: 'none',
    backgroundColor: getProvenanceHighlight(2),
    // use sl tooltip variables for consistancy
    fontFamily: 'var(--sl-tooltip-font-family)',
    borderRadius: 'var(--sl-tooltip-border-radius)',
    fontSize: 'var(--sl-tooltip-font-size)',
    fontWeight: 'var(--sl-tooltip-font-weight)',
    lineHeight: 'var(--sl-tooltip-line-height)',
    padding: 'var(--sl-tooltip-padding)',
  },
  'div.cm-tooltip-arrow::after': {
    borderBottomColor: `${getProvenanceHighlight(2)} !important`,
  },
  'div.cm-tooltip-arrow::before': {
    borderBottomColor: `${getProvenanceHighlight(2)} !important`,
  },
})

/**
 * Creates a set of codemirror mark type decorations from the
 * array of `ProvenanceMarkers`
 * @param marks `PorvenanceMarker[]`
 * @returns `DecorationSet`
 */
const createProvenanceDecorations = (marks: ProvenanceMarker[]) =>
  Decoration.set(
    marks.map((mark) => {
      return Decoration.mark({
        tagName: 'span',
        class: `prov-lvl-${mark.mi}`,
        attributes: {
          style:
            mark.mi >= 0 && mark.mi <= 5
              ? `background-color: ${getProvenanceHighlight(mark.mi as ProvenanceHighlightLevel)}`
              : '',
        },
      }).range(mark.from, mark.to)
    })
  )

const addTooltipContent = (dom: HTMLDivElement, content: string) => {
  const el = document.createElement('div')
  el.textContent = content
  dom.appendChild(el)
}

/**
 * Create a hover tooltip to display the authorship provenance information
 * @param marks `PorvenanceMarker[]`
 * @returns `Extension`
 */
const provenanceTooltip = (marks: ProvenanceMarker[]) =>
  hoverTooltip((_, pos) => {
    for (const mark of marks) {
      if (pos >= mark.from && pos <= mark.to) {
        return {
          pos,
          above: false,
          arrow: true,
          create: () => {
            const dom = document.createElement('div')
            dom.className = 'cm-provenance-tooltip'

            addTooltipContent(
              dom,
              `${mark.count} Author${mark.count > 1 ? 's' : ''}`
            )
            addTooltipContent(dom, mark.provenance)

            return { dom, offset: { x: 0, y: 10 } }
          },
        }
      }
    }

    return null
  })

export {
  executionMessageLinter,
  stencilaTheme,
  createProvenanceDecorations,
  provenanceTooltip,
}
