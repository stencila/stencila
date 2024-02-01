import { EditorView } from '@codemirror/view'

/**
 * Codemirror `Extension` to allow overriding default
 * editor styles.
 * use the class selector of the target element as the key
 * and add a `StyleSpec` object as the value
 */
const editorStyles = EditorView.baseTheme({
  '.cm-scroller': {
    maxWidth: '70%',
    height: 'content',
  },
})

export { editorStyles }
