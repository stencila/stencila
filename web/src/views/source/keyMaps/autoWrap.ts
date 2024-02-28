import { EditorSelection, TransactionSpec } from '@codemirror/state'
import { Command, KeyBinding, EditorView } from '@codemirror/view'

type TextWrapper = '()' | '[]' | '{}' | "''" | '``' | '""'

const wrappers: { [k: string]: TextWrapper } = {
  parenthesis: '()',
  bracket: '[]',
  curlyBrace: '{}',
  backTick: '``',
  singleQuote: "''",
  doubleQuote: '""',
}

/**
 * Returns a key-mapping event handler which will auto wrap selected content in the
 * chosen bracket char, or quote marks.
 * If no text is selected, it creates the closing char automatically, placing the cursor
 * in the middle
 *
 * @param wrapper the set of wrappers to be applied to the event handler
 * @returns `Command` function
 */
const autoWrapHandler =
  (wrapper: TextWrapper): Command =>
  (view: EditorView) => {
    const { state } = view
    if (state.selection.main.empty) {
      const cursor = state.selection.main.head
      let override = false
      let trSpec: TransactionSpec

      // handle quote mark behaviour differently
      // - if cursor pos is inbetween 2 quotes, move cursor offset along 1.
      // - else if cursor pos is before OR after a quote, revert to default.
      if (
        [
          wrappers.singleQuote,
          wrappers.doubleQuote,
          wrappers.backTick,
        ].includes(wrapper)
      ) {
        const prevChar = state.doc.sliceString(cursor - 1, cursor)
        const nextChar = state.doc.sliceString(cursor, cursor + 1)
        if (prevChar === wrapper[0] && nextChar === wrapper[1]) {
          trSpec = { selection: EditorSelection.single(cursor + 1) }
          override = true
        } else if (prevChar === wrapper[1] || nextChar === wrapper[0]) {
          return false
        }
      }
      if (!trSpec) {
        trSpec = {
          changes: {
            from: cursor,
            insert: wrapper[1],
          },
        }
      }
      const tr = view.state.update(trSpec)
      view.dispatch(tr)
      return override
    } else {
      const { from, to } = state.selection.main
      const tr = view.state.changeByRange(() => ({
        changes: [
          { from, insert: wrapper[0] },
          { from: to, insert: wrapper[1] },
        ],
        range: EditorSelection.range(from + 1, to + 1),
      }))
      view.dispatch(tr)
      return true
    }
  }

/**
 * A handler for the closing wrapper characters,
 * if cursor directly in between the chars, then it will move the cursor
 * 1 position, instead of creating a new closing char.
 * @param wrapper the set of wrappers to be applied to the event handler
 * @returns `Command` function
 */
const handleAutoWrapClose =
  (wrapper: TextWrapper): Command =>
  (view: EditorView) => {
    const { state } = view
    if (state.selection.main.empty) {
      const cursor = state.selection.main.head
      const prevChar = state.doc.sliceString(cursor - 1, cursor)
      const nextChar = state.doc.sliceString(cursor, cursor + 1)
      if (prevChar === wrapper[0] && nextChar === wrapper[1]) {
        view.dispatch(
          view.state.update({ selection: EditorSelection.single(cursor + 1) })
        )
        return true
      }
    }
    return false
  }

const autoWrapKeys: KeyBinding[] = [
  {
    key: '(',
    run: autoWrapHandler(wrappers.parenthesis),
  },
  {
    key: '[',
    run: autoWrapHandler(wrappers.bracket),
  },
  {
    key: '{',
    run: autoWrapHandler(wrappers.curlyBrace),
  },
  {
    key: '"',
    run: autoWrapHandler(wrappers.doubleQuote),
  },
  {
    key: "'",
    run: autoWrapHandler(wrappers.singleQuote),
  },
  {
    key: ')',
    run: handleAutoWrapClose(wrappers.parenthesis),
  },
  {
    key: ']',
    run: handleAutoWrapClose(wrappers.bracket),
  },
  {
    key: '}',
    run: handleAutoWrapClose(wrappers.curlyBrace),
  },
]

export { autoWrapKeys }
