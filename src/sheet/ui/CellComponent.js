import { isString, uuid } from 'substance'
import NodeComponent from '../../shared/NodeComponent'
import Cell from '../model/Cell'
import CellEditor from './CellEditor'
import ExpressionComponent from './ExpressionComponent'
import ConstantComponent from './ConstantComponent'
import PrimitiveComponent from './PrimitiveComponent'
import BooleanComponent from './BooleanComponent'
import ErrorComponent from './ErrorComponent'
import HTMLCellComponent from './HTMLCellComponent'
import ImageCellComponent from './ImageCellComponent'

export default
class CellComponent extends NodeComponent {

  getChildContext() {
    return {
      cell: this
    }
  }

  render($$) {
    const isEditing = this.isEditing()

    let el = $$('td').addClass('se-cell')
    el.addClass(isEditing ? 'sm-edit' : 'sm-display')
    if (isEditing) {
      el.append(this.renderEditor($$))
    } else {
      const node = this.getNode()
      if (!node) {
        el.addClass('sm-empty')
      } else {
        el.append(this.renderDisplay($$))
      }
      el.on('click', this.onClick)
      el.on('dblclick', this.onDblClick)
    }
    return el
  }

  renderEditor($$) {
    const node = this.getNode()
    let content
    if (isString(this.state.initialContent)) {
      content = this.state.initialContent
    } else if (node) {
      content = node.content
    } else {
      content = ''
    }
    return $$(CellEditor, {
      content: content,
      select: this.state.initialContent ? 'last' : 'all'
    }).ref('editor')
  }

  renderDisplay($$) {
    const node = this.getNode()
    // Display node content
    let CellContentClass
    if (node.isConstant()) {
      CellContentClass = ConstantComponent
    } else if (node.valueType) {
      switch (node.valueType) {
        // TODO: rethink the value type system
        case 'primitive':
          CellContentClass = PrimitiveComponent
          break
        case 'boolean':
          CellContentClass = BooleanComponent
          break
        case 'image':
          CellContentClass = ImageCellComponent
          break
        case 'html':
          CellContentClass = HTMLCellComponent
          break
        case 'error':
          CellContentClass = ErrorComponent
          break
        default:
          CellContentClass = ExpressionComponent
      }
    }
    return $$(CellContentClass, {node: node}).ref('content')
  }

  getNode() {
    return this.props.node
  }

  getDocument() {
    return this.context.editorSession.getDocument()
  }

  getEditorSession() {
    return this.context.editorSession
  }

  /**
    There are 3 alternative display modes for cells

    clipped: uses minimal space
    expanded: expands the height and width of the cell to display all content
    overlay: displays content in an overlay that covers other cells
  */
  toggleDisplayMode() {
    const node = this.props.node
    // empty cells do not have a node
    if (!node) return

    const currentMode = node.displayMode
    const content = this.refs.content
    if (content) {
      const nextMode = Cell.getNextDisplayMode(currentMode)
      const editorSession = this.getEditorSession()
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
    let editorSession = this.getEditorSession()
    const node = this.props.node
    const newContent = this.getCellEditorContent() || ''
    if (!node) {
      const row = this.getRow()
      const col = this.getCol()
      const id = uuid()
      editorSession.transaction((tx) => {
        tx.create({
          type: "sheet-cell",
          id: id,
          row: row,
          col: col,
          content: newContent
        })
      })
      // TODO: is this really necessary?
      // The new node should essiantly get propagated
      // by the parent
      // node = doc.get(id)
      // this.extendProps({ node: node })
    } else if (newContent !== node.content) {
      editorSession.transaction((tx) => {
        tx.set([node.id, 'content'], newContent)
      })
      // TODO: do we need the cached value?
      // if yes, it should not be managed here, but done
      // by the node itself
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

  getRow() {
    return parseInt(this.getAttribute('data-row'), 10)
  }

  getCol() {
    return parseInt(this.getAttribute('data-col'), 10)
  }

  getPos() {
    return {
      row: this.getRow(),
      col: this.getCol()
    }
  }

  onClick(e) {
    e.preventDefault()
    e.stopPropagation()
    this.send('selectCell', this)
  }

  onDblClick(e) {
    e.preventDefault()
    e.stopPropagation()
    this.enableEditing()
    this.send('activateCell', this)
  }
}

CellComponent.prototype._isCellComponent = true