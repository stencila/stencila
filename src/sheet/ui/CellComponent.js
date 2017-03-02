import { Component, isString, uuid } from 'substance'
import CellEditor from './CellEditor'
import ExpressionComponent from './ExpressionComponent'
import ConstantComponent from './ConstantComponent'

export default
class CellComponent extends Component {

  didMount() {
    this._connect()
  }

  dispose() {
    this._disconnect()
  }

  willReceiveProps() {
    this._disconnect()
  }

  didReceiveProps() {
    this._connect()
  }

  render($$) {
    var node = this.props.node
    var el = $$('td').addClass('se-cell')
    var componentRegistry = this.context.componentRegistry
    var isEditing = this.isEditing()
    el.addClass(isEditing ? 'sm-edit' : 'sm-display')

    if (isEditing) {
      var content
      if (isString(this.state.initialContent)) {
        content = this.state.initialContent
      } else if (node) {
        content = node.content
      } else {
        content = ''
      }
      el.append($$(CellEditor, {
        content: content,
        select: this.state.initialContent ? 'last' : 'all'
      }).ref('editor'))
    } else {
      el.on('dblclick', this.onDblClick)

      // Display node content
      if (node) {
        var CellContentClass
        if (node.isConstant()) {
          CellContentClass = ConstantComponent
        } else if (node.valueType) {
          CellContentClass = componentRegistry.get('cell:'+node.valueType, 'strict')
        }
        if (!CellContentClass) {
          CellContentClass = ExpressionComponent
        }
        var cellContentEl = $$(CellContentClass, {node: node}).ref('content')
        el.append(cellContentEl)

        el.addClass(node.getDisplayClass())

      }
    }
    return el
  }

  getNode() {
    return this.props.node
  }

  getDocument() {
    return this.context.doc
  }

  getDocumentSession() {
    return this.context.documentSession
  }

  /**
    There are 3 alternative display modes for cells

    clipped: uses minimal space
    expanded: expands the height and width of the cell to display all content
    overlay: displays content in an overlay that covers other cells
  */
  toggleDisplayMode() {
    var node = this.props.node
    // empty cells do not have a node
    if (!node) return

    var currentMode = node.displayMode
    var content = this.refs.content
    if (content) {
      var modes = ['cli', 'exp', 'ove']
      var idx = modes.indexOf(currentMode)+1
      if (modes.length > 0) {
        idx = idx%modes.length
      }
      var nextMode = modes[idx] || ''
      var editorSession = this.context.editorSession
      editorSession.transaction(function(tx) {
        tx.set([node.id, 'displayMode'], nextMode)
      })
    }
  }

  /**
    Ad-hoc creates a node when editing is enabled for an empty cell
  */
  enableEditing(initialContent) {
    this.extendState({
      edit: true,
      initialContent: initialContent
    })
  }

  commit() {
    var docSession = this.getDocumentSession()
    var doc = this.getDocument()
    var node = this.props.node
    var newContent = this.getCellEditorContent() || ''
    if (!node) {
      var row = parseInt(this.attr('data-row'), 10)
      var col = parseInt(this.attr('data-col'), 10)
      var id = uuid()
      docSession.transaction(function(tx) {
        tx.create({
          type: "sheet-cell",
          id: id,
          row: row,
          col: col,
          content: newContent
        })
      })
      node = doc.get(id)
      this.extendProps({ node: node })
    } else if (newContent !== node.content) {
      docSession.transaction(function(tx) {
        tx.set([this.props.node.id, 'content'], newContent)
      }.bind(this))
      delete this.props.node.value
    }
    this.extendState({ edit: false })
  }

  discard() {
    this.extendState({
      edit: false
    })
  }

  isEditing() {
    return this.state.edit
  }

  getCellEditorContent() {
    if (this.refs.editor) {
      return this.refs.editor.getContent()
    }
  }

  onDblClick(e) {
    e.preventDefault()
    e.stopPropagation()
    this.enableEditing()
    this.send('activateCurrentCell')
  }

  _connect() {
    const doc = this.getDocument()
    const node = this.props.node
    if (node) {
      doc.getEventProxy('path').connect(this, [node.id, 'content'], this.rerender)
      doc.getEventProxy('path').connect(this, [node.id, 'displayMode'], this.rerender)
      node.on('cell:changed', this._onCellChange, this)
    }
  }

  _onCellChange() {
    this.rerender()
  }

  _disconnect() {
    var doc = this.getDocument()
    if (this.props.node) {
      doc.getEventProxy('path').disconnect(this)
      this.props.node.off(this)
    }
  }

}