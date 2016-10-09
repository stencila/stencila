import Component from 'substance/ui/Component'

import code from '../../utilities/code'

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

  constructor(...args) {
    super(...args)

    this.editor = null
    this.editorMute = false

    // In `this._onCodeChanged` and `this._onLanguageChanged`, these custom props
    // are not on `this.props` for some reason. So, "store" them here.
    this.codeProperty = this.props.codeProperty
    this.languageProperty = this.props.languageProperty
  }

  render ($$) {
    var node = this.props.node
    return $$('div')
      .addClass('sc-code-editor')
      .append(
        $$('div')
          .ref('editor')
          .text(node[this.props.codeProperty])
      )
  }

  didMount () {
    var node = this.props.node

    // Resolve the language for the code
    var language
    if (this.props.languageProperty) {
      language = node[this.props.languageProperty]
    } else {
      language = this.props.language
    }

    // Attach ACE editor (allows for asynchronous loading of ACE)
    code.attachAceEditor(
      this.refs.editor.getNativeElement(),
      node[this.props.codeProperty],
      {
        language: language,
        fontSize: 15,
        // FIXME
        // This does not update when the editor state is changed (e.g editing turned from off to on)
        // Probably needs a custom event like `_onContentChanged` below
        readOnly: false//! this.context.controller.state.edit
      },
      function (editor) {
        // When editor has been created...

        // For consistency and simplicity use single character newlines
        editor.getSession().setNewLineMode('unix')

        // Additional options
        // ESC keypress
        editor.commands.addCommand({
          name: 'escape',
          bindKey: {win: 'Escape', mac: 'Escape'},
          exec: function (editor) {
            this.send('escape')
            editor.blur()
          }.bind(this),
          readOnly: true
        })

        // editor.on('blur', this._onEditorBlur.bind(this));
        editor.on('change', this._onEditorChange.bind(this))
        this.editor = editor
      }.bind(this)
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
  }

  /**
   * When there is a change in the editor, convert the change into a Substance change
   */
  _onEditorChange (change) {
    if (this.editorMute) return

    // For determining the position of changes...
    var length = function (lines) {
      var chars = 0
      for (var i = 0, l = lines.length; i < l; i++) {
        chars += lines[i].length
      }
      return chars + (lines.length - 1)
    }

    // Get the start position of the change
    var start = 0
    if (change.start.row > 0) {
      start = length(this.editor.getSession().getLines(0, change.start.row - 1)) + 1
    }
    start += change.start.column

    // Apply as a Substance update to the code property of the node
    var surface = this.context.surface
    var node = this.props.node
    var codeProperty = this.codeProperty
    if (change.action === 'insert') {
      surface.transaction(function (tx) {
        tx.update([node.id, codeProperty], {
          insert: {
            offset: start,
            value: change.lines.join('\n')
          }
        })
      }, {
        source: this,
        skipSelection: true
      })
    } else if (change.action === 'remove') {
      surface.transaction(function (tx) {
        tx.update([node.id, codeProperty], {
          delete: {
            start: start,
            end: start + length(change.lines)
          }
        })
      }, {
        source: this,
        skipSelection: true
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
      // Ignore editor chnage events
      this.editorMute = true

      var session = this.editor.getSession()
      var doc = session.getDocument()

      // Function to convert Substance offset to an Ace row/column position
      var offsetToPos = function (offset) {
        var row = 0
        var col = offset
        doc.getAllLines().forEach(function (line) {
          if (col <= line.length) {
            return
          }
          row += 1
          col -= line.length + 1
        })
        return {
          row: row,
          column: col
        }
      }

      // Apply each change
      change.ops.forEach(function (op) {
        var diff = op.diff
        if (diff.type === 'insert') {
          doc.insert(
            offsetToPos(diff.pos),
            diff.str
          )
        } else if (diff.type === 'delete') {
          // FIXME Deletion is not working properly and triggers the `setValue` fallback below
          var Range = window.ace.require('ace/range').Range
          doc.remove(Range.fromPoints(
            offsetToPos(diff.pos),
            offsetToPos(diff.pos + diff.str.length)
          ))
        } else {
          throw new Error('Unhandled diff:' + JSON.stringify(diff))
        }
      })

      // Check that editor text is what it should be
      var editorText = this.editor.getValue()
      var nodeText = this.props.node[codeProperty]
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

export default CodeEditorComponent
