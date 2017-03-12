import { Component } from 'substance'
import ace from 'brace'
import code from '../../utilities/code/index'

/**
 * A `Component` for editing a node's `source` code
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
class CodeEditorComponent extends Component {

  constructor (...args) {
    super(...args)

    this.editor = null
    this.editorMute = false

    // In `this._onCodeChanged` and `this._onLanguageChanged`, these custom props
    // are not on `this.props` for some reason. So, "store" them here.
    this.codeProperty = this.props.codeProperty
    this.languageProperty = this.props.languageProperty
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

  didMount () {
    const node = this.props.node

    // Resolve the language for the code
    let language
    if (this.props.languageProperty) {
      language = node[this.props.languageProperty]
    } else {
      language = this.props.language
    }

    // Attach ACE editor (allows for asynchronous loading of ACE)
    code.attachAceEditor(
      this.refs.editor.getNativeElement(),
      this._getCode(),
      {
        language: language,
        fontSize: 12,
        // FIXME
        // This does not update when the editor state is changed (e.g editing turned from off to on)
        // Probably needs a custom event like `_onContentChanged` below
        readOnly: false
      },
      (editor) => {
        // When editor has been created...

        // For consistency and simplicity use single character newlines
        editor.getSession().setNewLineMode('unix')

        // Additional options
        // ESC keypress
        editor.commands.addCommand({
          name: 'escape',
          bindKey: {win: 'Escape', mac: 'Escape'},
          exec: function (editor) {
            this.el.emit('escape')
            editor.blur()
          }.bind(this),
          readOnly: true
        })

        editor.commands.addCommand({
          name: 'update',
          bindKey: {win: 'Shift+Enter', mac: 'Shift+Enter'},
          exec: () => {
            this._setSourceCode()
          },
          readOnly: true
        })

        // The `_onEditorChange` method is better in that it allows for realtime collab
        // of code editors. But it is currently causing problems so using `_onEditorBlur` for now.
        editor.on('change', this._onEditorChange.bind(this))
        // editor.on('blur', this._onEditorBlur.bind(this))

        this.editor = editor
      }
    )

    node.on(this.props.codeProperty + ':changed', this._onCodeChanged, this)
    if (this.props.languageProperty) node.on(this.props.languageProperty + ':changed', this._onLanguageChanged, this)
  }

  shouldRerender () {
    // Don't rerender as that would destroy editor
    return false
  }

  dispose () {
    this.props.node.off(this)
    this.editor.destroy()
    this.editor = null
  }

  _getCode() {
    return this.props.node[this.props.codeProperty]
  }

  _setSourceCode() {
    const editorSession = this.context.editorSession
    editorSession.transaction(tx => {
      tx.set([this.props.node.id, this.codeProperty], this.editor.getValue())
    })
  }

  /**
   * When there is a change in the editor, convert the change into a Substance change
   */
  _onEditorChange (change) {
    // this guard is enabled while we apply changes received
    // from the editor session
    if (this.editorMute) return

    // Get the start position of the change
    let start = 0
    if (change.start.row > 0) {
      const lines = this.editor.getSession().getLines(0, change.start.row-1)
      start = countCharacters(lines)
    }
    start += change.start.column

    // Apply as a Substance update to the code property of the node
    const editorSession = this.context.editorSession
    const node = this.props.node
    var codeProperty = this.codeProperty
    if (change.action === 'insert') {
      editorSession.transaction((tx) => {
        tx.update([node.id, codeProperty], {
          type: 'insert',
          start: start,
          text: change.lines.join('\n')
        })
      }, {
        // leaving a trace here so we can skip the change in _onCodeChanged
        source: this,
        // tell EditorSession not to rerender the selection
        skipSelectionRerender: true
      })
    } else if (change.action === 'remove') {
      editorSession.transaction(function (tx) {
        tx.update([node.id, codeProperty], {
          type: 'delete',
          start: start,
          end: start + countCharacters(change.lines)-1
        })
      }, {
        source: this,
        skipSelectionRerender: true
      })
    } else {
      throw new Error('Unhandled change:' + JSON.stringify(change))
    }
  }

  /**
   * When the node's code property changes, update the
   * editor (if this wasn't the source of the update) by translating
   * the Substance change into an Ace change
   */
  _onCodeChanged (change, info) {
    var codeProperty = this.codeProperty
    if (info.source !== this && this.editor) {
      // Ignore editor change events
      this.editorMute = true

      const node = this.props.node
      const session = this.editor.getSession()
      const aceDoc = session.getDocument()

      // Apply each change
      change.ops.forEach(function (op) {
        if (op.type !== 'update' || op.path[0] !== node.id) return
        var diff = op.diff
        if (diff.type === 'insert') {
          aceDoc.insert(
            offsetToPos(aceDoc.getAllLines(), diff.pos),
            diff.str
          )
        } else if (diff.type === 'delete') {
          var Range = ace.acequire('ace/range').Range
          aceDoc.remove(Range.fromPoints(
            offsetToPos(aceDoc.getAllLines(), diff.pos),
            offsetToPos(aceDoc.getAllLines(), diff.pos + diff.str.length)
          ))
        } else {
          throw new Error('Unhandled diff:' + JSON.stringify(diff))
        }
      })

      // Check that editor text is what it should be
      const editorText = this.editor.getValue()
      const nodeText = this.props.node[codeProperty]
      if (editorText !== nodeText) {
        console.error('Code editor content does not match node code content. Falling back to `setValue`')
        this.editor.setValue(nodeText, -1)
      }

      // No longer ignore editor events
      this.editorMute = false
    }
  }

  /**
   * When the node's language changes, update the
   * editor (if this wasn't the source of the update)
   */
  _onLanguageChanged (change, info) {
    var languageProperty = this.languageProperty
    if (info.source !== this && this.editor) {
      code.setAceEditorMode(this.editor, this.props.node[languageProperty])
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
