import { isString } from 'substance'
import NodeComponent from '../../shared/NodeComponent'
import SheetCellEditor from './SheetCellEditor'
import ExpressionComponent from './ExpressionComponent'
import ConstantComponent from './ConstantComponent'

export default
class SheetCellComponent extends NodeComponent {

  getChildContext() {
    return {
      cell: this
    }
  }

  didMount() {
    super.didMount()

    const node = this.props.node
    if (node) {
      node.on('value:updated', this.rerender, this)
    }
  }

  dispose() {
    super.dispose()

    const node = this.props.node
    if (node) {
      node.off(this)
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
    return $$(SheetCellEditor, {
      content: content,
      select: this.state.initialContent ? 'last' : 'all'
    }).ref('editor')
  }

  renderDisplay($$) {
    const node = this.getNode()
    // Display node content
    let CellContentClass
    if (node.isLiteral()) {
      CellContentClass = ConstantComponent
    } else {
      CellContentClass = ExpressionComponent
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
    Ad-hoc creates a node when editing is enabled for an empty cell
  */
  enableEditing(initialContent) {
    this.extendState({
      edit: true,
      initialContent: initialContent
    })
  }

  disableEditing() {
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

SheetCellComponent.prototype._isCellComponent = true