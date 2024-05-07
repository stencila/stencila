import { linter, Diagnostic } from '@codemirror/lint'
import { Extension } from '@codemirror/state'
import { EditorView, Decoration } from '@codemirror/view'
import { MessageLevel } from '@stencila/types'

import { ExecutionMessage } from '../execution-message'

import type { ProvenanceMarker } from './types'

/**
 * Convert the `ExecutionMessage.level` value into a @codemirror/lint `Severity` string
 * for the linting extension
 * @param lvl `MessageLevel`
 * @returns 'Severity'
 */
const getMessageSeverity = (lvl: MessageLevel) =>
  lvl === 'Error'
    ? 'error'
    : // @ts-expect-error "Warning is not declared in the `MessageLevel` type, yet is sometimes used"
      lvl === 'Warning' || lvl === 'Warn'
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
      }).range(mark.from, mark.to)
    })
  )

const provTheme = EditorView.theme({
  '.prov-lvl-0': {
    backgroundColor: 'transparent',
  },
  '.prov-lvl-1': {
    backgroundColor: '#f1f5fe', // blue-50
  },
  '.prov-lvl-2': {
    backgroundColor: '#dbeafe', // blue-100
  },
  '.prov-lvl-3': {
    backgroundColor: '#bfdbfe', // blue-200
  },
  '.prov-lvl-4': {
    backgroundColor: '#93c5fd', // blue-300
  },
  '.prov-lvl-5': {
    backgroundColor: '#60a5fa', // blue-400
  },
})

export {
  executionMessageLinter,
  messagesTheme,
  createProvenanceDecorations,
  provTheme,
}
