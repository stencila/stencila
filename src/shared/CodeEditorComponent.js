import { CustomSurface, inBrowser, isNil, isArrayEqual } from 'substance'
import ace from 'brace'
import { attachAceEditor, setAceEditorMode } from '../utilities/aceHelpers'

/**
 * A `CustomSurface` for editing a node's `source` code
 * attribute.
 *
 * This is based on ideas here:
 *
 *   - https://github.com/substance/examples/blob/v1.0.0-beta.4/code-editor/script/ScriptEditor.js.
 *   - https://github.com/mizzao/meteor-sharejs/blob/master/sharejs-ace/ace.js
 *   - https://github.com/mixxen/share-ace/blob/master/share-ace.js
 *
 * @class      CodeEditorComponent (name)
 */
class CodeEditorComponent extends CustomSurface {

  constructor (...args) {
    super(...args)

    this.aceEditor = null
    this._editorMute = false
  }

  didMount () {
    super.didMount()

    if (inBrowser) {
      this._createAceEditor()
    }
  }

  dispose () {
    super.dispose()

    const editorSession = this.context.editorSession
    editorSession.off(this)
    if (this.aceEditor) {
      this.aceEditor.destroy()
      this.aceEditor = null
    }
  }

  render ($$) {
    return $$('div')
      .addClass('sc-code-editor')
      .append(
        $$('pre')
          .ref('editor')
          .text(this._getCode())
      )
  }

  shouldRerender() {
    // Don't rerender as that would destroy editor
    return false
  }

  setLanguage(language) {
    if (this.aceEditor) {
      setAceEditorMode(this.aceEditor, language)
    }
  }

  _focus() {
    if (this.aceEditor) {
      this.aceEditor.focus()
    }
  }

  // CustomSurface interface
  _getCustomResourceId() {
    return this.props.path.join('.')
  }

  _createAceEditor() {
    // Resolve the language for the code
    const language = this.props.language
    // Attach ACE editor (allows for asynchronous loading of ACE)
    attachAceEditor(
      this.refs.editor.getNativeElement(),
      this._getCode(),
      {
        language: language,
        fontSize: 13,
        fontFamily: 'Monaco, Menlo, "Ubuntu Mono", Consolas, "Source Code Pro", monospace',
        // FIXME
        // This does not update when the editor state is changed (e.g editing turned from off to on)
        // Probably needs a custom event like `_onContentChanged` below
        readOnly: false
      },
      (aceEditor) => {
        // When the ace editor is ready...
        this._onEditorCreated(aceEditor)
      }
    )
  }

  _onEditorCreated(aceEditor) {
    // For consistency and simplicity use single character newlines
    aceEditor.getSession().setNewLineMode('unix')
    // Additional options
    aceEditor.commands.addCommand({
      name: 'escape',
      bindKey: {win: 'Escape', mac: 'Escape'},
      exec: (aceEditor) => {
        this._onEscape(aceEditor)
      },
      readOnly: true
    })
    aceEditor.commands.addCommand({
      name: 'update',
      bindKey: {win: 'Shift+Enter', mac: 'Shift+Enter'},
      exec: (aceEditor) => {
        this._onConfirm(aceEditor)
      },
      readOnly: true
    })
    aceEditor.commands.addCommand({
      name: 'break',
      bindKey: {win: 'Alt+Enter', mac: 'Alt+Enter'},
      exec: () => {
        this.send('break')
      },
      readOnly: true
    })
    aceEditor.commands.addCommand({
      name: 'execute',
      bindKey: {win: 'Ctrl+Enter', mac: 'Ctrl+Enter'},
      exec: (aceEditor) => {
        this._onExecute(aceEditor)
      },
      readOnly: true
    })
    // The `_onEditorChange` method is better in that it allows for realtime collab
    // of code editors. But it is currently causing problems so using `_onEditorBlur` for now.
    aceEditor.on('change', this._onEditorChange.bind(this))
    aceEditor.on('focus', this._onEditorFocus.bind(this))

    if (this.props.lineNumbers === false) {
      aceEditor.renderer.setOption('showLineNumbers', false);
    }

    // Finally store the editor and register for document changes
    // not produced by this editor
    this.aceEditor = aceEditor
    const editorSession = this.context.editorSession
    editorSession.on('render', this._onCodeChanged, this, {
      resource: 'document',
      path: this.props.path
    })
  }

  _getDocument() {
    return this.context.editorSession.getDocument()
  }

  _getCode() {
    return this._getDocument().get(this.props.path)
  }

  /**
   * When there is a change in the editor, convert the change into a Substance change
   */
  _onEditorChange(change) {
    // this guard is enabled while we apply changes received
    // from the editor session
    if (this._editorMute) return

    const editorSession = this.context.editorSession
    const path = this.props.path
    const aceEditor = this.aceEditor

    // Get the start position of the change
    let start = 0
    if (change.start.row > 0) {
      const lines = aceEditor.getSession().getLines(0, change.start.row-1)
      start = countCharacters(lines)
    }
    start += change.start.column

    // Apply as a Substance update to the code property of the node
    if (change.action === 'insert') {
      editorSession.transaction((tx) => {
        const code = tx.get(path)
        if (isNil(code)) {
          tx.set(path, change.lines.join('\n'))
        } else {
          tx.update(path, {
            type: 'insert',
            start: start,
            text: change.lines.join('\n')
          })
        }
        // TODO: put ace selection data here, so that
        // we can recover the selection
        tx.selection = {
          type: 'custom',
          surfaceId: this.getId(),
          customType: 'ace'
        }
      }, {
        // leaving a trace here so we can skip the change in _onCodeChanged
        source: this
      })
    } else if (change.action === 'remove') {
      editorSession.transaction((tx) => {
        tx.update(path, {
          type: 'delete',
          start: start,
          end: start + countCharacters(change.lines)-1
        })
        // TODO: put ace selection data here, so that
        // we can recover the selection
        tx.selection = {
          type: 'custom',
          surfaceId: this.getId(),
          customType: 'ace'
        }
      }, {
        source: this
      })
    } else {
      throw new Error('Unhandled change:' + JSON.stringify(change))
    }
  }

  _onEditorFocus() {
    // ATTENTION: already tried to set the selection on focus,
    // but this is gets triggered in too many cases, causing troubles
  }

  /**
   * When the node's code property changes, update the
   * editor (if this wasn't the source of the update) by translating
   * the Substance change into an Ace change
   */
  _onCodeChanged(change, info) {
    if (info.source === this || !this.aceEditor) return

    const aceEditor = this.aceEditor
    const session = aceEditor.getSession()
    const aceDoc = session.getDocument()
    const doc = this.context.editorSession.getDocument()
    const path = this.props.path
    const code = doc.get(path)

    // Ignore editor change events
    this._editorMute = true
    function _applyChange(op) {
      if (op.type === 'update') {
        const diff = op.diff
        switch(diff.type) {
          case 'insert': {
            aceDoc.insert(
              offsetToPos(aceDoc.getAllLines(), diff.pos),
              diff.str
            )
            break
          }
          case 'delete': {
            var Range = ace.acequire('ace/range').Range
            aceDoc.remove(Range.fromPoints(
              offsetToPos(aceDoc.getAllLines(), diff.pos),
              offsetToPos(aceDoc.getAllLines(), diff.pos + diff.str.length)
            ))
            break
          }
          default:
            console.error('Unhandled diff:', JSON.stringify(diff))
        }
      } else if (op.type === 'set') {
        aceEditor.setValue(op.val, -1)
      }
    }
    change.ops
      .filter(op => isArrayEqual(op.path, path))
      .forEach(_applyChange)

    // Check that editor text is what it should be
    const editorText = aceEditor.getValue()
    if (editorText !== code) {
      console.error('Code editor content does not match node code content. Falling back to `setValue`')
      aceEditor.setValue(code, -1)
    }
    // No longer ignore editor events
    this._editorMute = false
  }

  _onEscape(editor) {
    this.el.emit('escape')
    editor.blur()
  }

  _onConfirm(editor) {
    const editorSession = this.context.editorSession
    editorSession.transaction(tx => {
      tx.set(this.props.path, editor.getValue())
    })
  }

  _onExecute(editor) {
    const editorSession = this.context.editorSession
    const doc = editorSession.getDocument()
    const src = doc.get(this.props.path)
    const newSrc = editor.getValue()
    if (src !== newSrc) {
      // changing the source will also trigger re-evaluation
      editorSession.transaction(tx => {
        tx.set(this.props.path, newSrc)
      })
    } else {
      this.send('execute')
    }
  }

}

CodeEditorComponent.fullWidth = true

// For determining the position of changes...
function countCharacters (lines) {
  // number of characters is the length of each lines plus line-endings
  return lines.reduce((sum, l) => { return sum + l.length }, 0)+lines.length
}

// Function to convert Substance offset to an Ace row/column position
function offsetToPos (lines, offset) {
  let row = 0
  let col = offset
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (col <= line.length) break
    row += 1
    col -= line.length + 1
  }
  return { row, col }
}

export default CodeEditorComponent
