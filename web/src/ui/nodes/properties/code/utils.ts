import { linter, Diagnostic } from '@codemirror/lint'
import { Extension } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { MessageLevel } from '@stencila/types'

import { ExecutionMessage } from '../execution-message'

/**
 * Convert the `ExecutionMessage.level` value into a @codemirror/lint `Severity` string
 * for the linting extension
 * @param lvl `MessageLevel`
 * @returns 'Severity'
 */
const getMessageSeverity = (lvl: MessageLevel) =>
  // @ts-expect-error "Exception is not declared in the `MessageLevel` type, yet is sometimes used"
  lvl === 'Error' || lvl === 'Exception'
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

export { executionMessageLinter, messagesTheme }
