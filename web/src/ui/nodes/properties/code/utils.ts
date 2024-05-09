import { linter, Diagnostic } from '@codemirror/lint'
import { Extension } from '@codemirror/state'
import { EditorView, Decoration } from '@codemirror/view'
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

const messagesTheme = EditorView.theme({
  '.cm-diagnostic': {
    fontFamily: 'mono',
    paddingLeft: '16px',
    paddingRight: '16px',
    borderBottom: '1px solid #dedede', // grey-200
  },
  '.cm-diagnostic:last-child': {
    borderBottom: '0px',
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

export { executionMessageLinter, messagesTheme, createProvenanceDecorations }
