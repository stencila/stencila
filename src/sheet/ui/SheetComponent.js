import {
  Component, DefaultDOMElement,
  inBrowser, uuid, findParentComponent
} from 'substance'
import {getColumnName} from '../model/sheetHelpers'
import SheetEngine from '../model/SheetEngine'
import SheetCellComponent from './SheetCellComponent'

export default
class SheetComponent extends Component {

  constructor(...args) {
    super(...args)

    this.id = uuid()

    this.handleActions({
      'selectCell': this.selectCell,
      // called after finishing editing a cell
      'commitCellChange': this.commitCellChange,
      'discardCellChange': this.discardCellChange,
      // called when double clicking on a cell
      'activateCell': this.activateCell
    })

    // internal state flags
    // set when inside a cell
    // TODO: we should try to find a better way to derive
    // this state more implicitly. ATM we need to invalidate
    // _activeCellComp whenever the cell editor is 'blurred'
    this._activeCellComp = null
    // flag indicating that the selection is inside this sheet
    this._hasSelection = false
    // used while creating a selection
    this._isSelecting = false
    this._startCell = null
    this._endCell = null

    this._sheetEngine = new SheetEngine(this.context.editorSession, this.props.node)

    // binding this, as these handlers are attached to global DOM elements
    // this.onGlobalKeydown = this.onGlobalKeydown.bind(this)
    // this.onGlobalKeypress = this.onGlobalKeypress.bind(this)
    // this.onWindowResize = this.onWindowResize.bind(this)
  }

  didMount() {
    const editorSession = this.context.editorSession
    editorSession.on('render', this.onSelectionChange, this, {
      resource: 'selection'
    })

    const globalEventHandler = this.context.globalEventHandler
    globalEventHandler.on('keydown', this.onGlobalKeydown, this, { id: this.id })
    globalEventHandler.on('keypress', this.onGlobalKeypress, this, { id: this.id })

    if (inBrowser) {
      window.addEventListener('resize', this.onWindowResize, false)
    }
  }

  dispose() {
    const editorSession = this.context.editorSession
    editorSession.off(this)

    const globalEventHandler = this.context.globalEventHandler
    globalEventHandler.off(this)

    if (inBrowser) {
      window.removeEventListener('resize', this.onWindowResize)
    }
  }

  render($$) {
    var el = $$('div').addClass('sc-sheet')
    el.append(
      this.renderTable($$)
    )
    el.append(
      $$('div').addClass('se-selection').ref('selection')
    )
    el.on('mousedown', this.onMouseDown)
    el.on('mouseover', this.onMouseOver)

    return el
  }

  renderTable($$) {
    // TODO: this code is almost identical to the exporter
    // we should try to share the code
    const sheet = this.props.node

    // 52 = 2*26 ~ A - AZ
    const ncols = Math.max(52, sheet.getColumnCount())
    const nrows = Math.max(100, sheet.getRowCount())
    const tableEl = $$('table').addClass("sc-sheet")

    // create header row
    const thead = $$('thead')
    const exprBar = $$('tr').addClass('se-expression-bar')
    exprBar.append(
      $$('th').addClass('se-label').html('f<sub>x</sub>'),
      $$('td').addClass('se-expression-field').attr('colspan', ncols-1).append(
        // TODO: add a TextInput, and think how we can 're-use'
        // it within the cell, so that changes are visible at both places.
        // $$(TextInput, {
        //   content: this.props.content
        // }).ref('expression')
        //   .on('confirm', this.onConfirmExpression)
        //   .on('cancel', this.onCancelExpression)
      )
    )
    thead.append(exprBar)
    const headerRow = $$('tr').addClass('se-header-row')
    headerRow.append($$('th').addClass('se-cell'))
    for (let j = 0; j < ncols; j++) {
      headerRow.append($$('th').text(
        getColumnName(j)
      ).addClass('se-cell'))
    }
    thead.append(headerRow)
    tableEl.append(thead)

    const tbody = $$('tbody').ref('body')
    for (let i = 0; i < nrows; i++) {
      const rowEl = $$('tr').attr('data-row', i).addClass('se-row')
      // first column is header
      rowEl.append(
        $$('th').addClass('se-cell').text(String(i+1))
      )
      // render all cells
      for (let j = 0; j < ncols; j++) {
        const cell = sheet.getCellAt(i, j)
        // Render Cell content
        rowEl.append(
          $$(SheetCellComponent, { node: cell })
            .attr('data-row', i)
            .attr('data-col', j)
        )
      }
      tbody.append(rowEl)
    }
    tableEl.append(tbody)
    return tableEl
  }

  getSelection() {
    let sel = this.context.editorSession.getSelection()
    if (sel.isCustomSelection() &&
      sel.customType === 'table' &&
      sel.data.sheetId === this.props.node.id
    ) {
      return sel
    }
  }

  getEditorSession() {
    return this.context.editorSession
  }

  getSheet() {
    return this.props.node
  }

  setSelection(tableSel) {
    // console.log('setSelection', tableSel)
    this._ensureActiveCellIsCommited()
    this._setSelection(tableSel)
  }

  isEditing() {
    const activateCell = this._activeCellComp
    return (activateCell && activateCell.isEditing())
  }

  // Action handlers

  selectCell(cellComp) {
    this._ensureActiveCellIsCommited()
    this._selectCell(cellComp)
  }

  activateCell(cellComp, initialContent) {
    this._ensureActiveCellIsCommited(cellComp)
    this._activeCellComp = cellComp
    cellComp.enableEditing(initialContent)
    // hiding the selection, as it is not good to have
    // an overlay while the context is changed
    this._hideSelection()
  }

  commitCellChange(content, key) {
    if (!this._activeCellComp) {
      console.warn('FIXME: expected to have an active cell.')
    } else {
      const cellComp = this._activeCellComp
      this._activeCellComp = null
      this._commitCell(cellComp)
    }
    if (key === 'enter') {
      this._selectNextCell(1, 0)
    }
  }

  discardCellChange() {
    const cellComp = this._activeCellComp
    this._activeCellComp = null
    cellComp.discard()
    this._selectCell(cellComp)
  }

  // Events

  onSelectionChange(sel) {
    // console.log('### ', sel)
    if (sel.isCustomSelection() &&
      sel.customType === 'table' &&
      sel.data.sheetId === this.props.node.id)
    {
      this._hasSelection = true
      this._rerenderSelection(sel)
    } else {
      this._hasSelection = false
      this._hideSelection()
    }
  }

  onMouseDown(event) {
    let target = findParentComponent(event.target)
    let cell = target._isCellComponent ? target : target.context.cell
    // happens when not on a cell, e.g. on the header
    if (!cell) {
      // console.log('not on a cell')
      return
    }
    if (cell.isEditing()) {
      // console.log('cell is editing')
      return
    }
    // only enable cell selection on cells which are not currently edited
    event.preventDefault()
    event.stopPropagation()
    this._isSelecting = true
    this.el.getOwnerDocument().on('mouseup', this.onMouseUp, this, { once: true })
    this._startCell = cell
    this._endCell = cell
    // this._ensureActiveCellIsCommited()
    this._updateSelection()
  }

  onMouseOver(event) {
    if (!this._isSelecting) return
    let target = findParentComponent(event.target)
    let cell = target._isCellComponent ? target : target.context.cell
    if (this._endCell !== cell) {
      this._endCell = cell
      this._updateSelection()
    }
  }

  onMouseUp() {
    if (this._isSelecting) {
      const start = this._startCell
      const end = this._endCell
      this._isSelecting = false
      this._startCell = null
      this._endCell = null
      if (start !== end) {
        this._updateSelection()
      }
    }
  }

  /*
    Will be bound to body element to receive events while not
    editing a cell.
    Note: these need to be done on keydown to prevent default browser
    behavior.
  */
  onGlobalKeydown(event) {
    // console.log('onGlobalKeydown()', 'keyCode=', event.keyCode)
    var handled = false

    if (!this.isEditing()) {
      // LEFT
      if (event.keyCode === 37) {
        if (event.shiftKey) {
          this._expandSelection(0, -1)
        } else {
          this._selectNextCell(0, -1)
        }
        handled = true
      }
      // RIGHT
      else if (event.keyCode === 39) {
        if (event.shiftKey) {
          this._expandSelection(0, 1)
        } else {
          this._selectNextCell(0, 1)
        }
        handled = true
      }
      // UP
      else if (event.keyCode === 38) {
        if (event.shiftKey) {
          this._expandSelection(-1, 0)
        } else {
          this._selectNextCell(-1, 0)
        }
        handled = true
      }
      // DOWN
      else if (event.keyCode === 40) {
        if (event.shiftKey) {
          this._expandSelection(1, 0)
        } else {
          this._selectNextCell(1, 0)
        }
        handled = true
      }
      // ENTER
      else if (event.keyCode === 13) {
        if (this.getSelection().isCollapsed()) {
          this._activateCurrentCell()
        }
        handled = true
      }
      // BACKSPACE | DELETE
      else if (event.keyCode === 8 || event.keyCode === 46) {
        this._deleteSelection()
        handled = true
      }
      // undo/redo
      else if (event.keyCode === 90 && (event.metaKey||event.ctrlKey)) {
        if (event.shiftKey) {
          this.getController().executeCommand('redo')
        } else {
          this.getController().executeCommand('undo')
        }
        handled = true
      }
    }

    if (handled) {
      // console.log('SheetEditor.onGlobalKeydown() handled event', event)
      event.stopPropagation()
      event.preventDefault()
    }
  }

  /*
    Will be bound to body element to receive events while not
    editing a cell.
    Note: only 'keypress' allows us to detect key events which
    would result in content changes.
  */
  onGlobalKeypress(event) {
    // console.log('onGlobalKeypress()', 'keyCode=', event.keyCode)
    var handled = false

    if (!this.isEditing()) {
      var character = String.fromCharCode(event.charCode)
      if (character) {
        this._activateCurrentCell(character)
        handled = true
      }
    }

    if (handled) {
      event.stopPropagation()
      event.preventDefault()
    }
  }

  onWindowResize() {
    this._rerenderSelection()
  }

  onConfirmExpression() {
    console.info('Confirmed expression change')
  }

  onCancelExpression() {
    console.info('Canceled expression change')
  }

  // private API

  _setSelection(tableSel) {
    const editorSession = this.context.editorSession
    // TODO: Make it easier to use custom selections.
    tableSel.sheetId = this.props.node.id
    const selData = {
      type: 'custom',
      customType: 'table',
      data: tableSel,
      surfaceId: this.id,
    }
    if (this.context.surface) {
      const surface = this.context.surface
      selData.surfaceId = surface.id
      if (surface.isContainerEditor()) {
        selData.containerId = surface.getContainerId()
      }
    }
    // console.log('_setSelection', selData)
    editorSession.setSelection(selData)
  }

  _selectCell(cellComp) {
    let row = cellComp.getRow()
    let col = cellComp.getCol()
    this._setSelection({
      startRow: row,
      startCol: col,
      endRow: row,
      endCol: col
    })
  }

  _commitCell(cellComp) {
    const editorSession = this.getEditorSession()
    const row = cellComp.getRow()
    const col = cellComp.getCol()
    const newContent = cellComp.getCellEditorContent()
    editorSession.transaction((tx) => {
      let sheet = tx.get(this.props.node.id)
      sheet.updateCell(row, col, newContent)
    })
    let cell = this.props.node.getCellAt(row, col)
    cellComp.disableEditing()
    cellComp.setProps({ node: cell })
  }

  _ensureActiveCellIsCommited(cellComp) {
    if (this._activeCellComp && this._activeCellComp !== cellComp) {
      this._commitCell(this._activeCellComp)
    }
    this._activeCellComp = null
  }

  _updateSelection() {
    if (this._startCell) {
      const startPos = this._startCell.getPos()
      const endPos = this._endCell.getPos()
      const newSel = {}
      newSel.startRow = Math.min(startPos.row, endPos.row)
      newSel.startCol = Math.min(startPos.col, endPos.col)
      newSel.endRow = Math.max(startPos.row, endPos.row)
      newSel.endCol = Math.max(startPos.col, endPos.col)
      this.setSelection(newSel)
    }
  }

  _selectNextCell(rowDiff, colDiff) {
    const sel = this.getSelection()
    if (!sel) return
    let data = Object.assign({}, sel.data)
    data.startRow = data.startRow + rowDiff
    // TODO: also ensure upper bound
    if (rowDiff < 0) {
      data.startRow = Math.max(0, data.startRow)
    }
    data.endRow = data.startRow
    data.startCol = data.startCol + colDiff
    // TODO: also ensure upper bound
    if (colDiff < 0) {
      data.startCol = Math.max(0, data.startCol)
    }
    data.endCol = data.startCol
    this.setSelection(data)
  }

  _expandSelection(rowDiff, colDiff) {
    const sel = this.getSelection()
    if (!sel) return
    let data = Object.assign({}, sel.data)

    data.endRow = data.endRow + rowDiff
    // TODO: also ensure upper bound
    if (rowDiff < 0) {
      data.endRow = Math.max(0, data.endRow)
    }

    data.endCol = data.endCol + colDiff
    // TODO: also ensure upper bound
    if (colDiff < 0) {
      data.endCol = Math.max(0, data.endCol)
    }

    data.startRow = data.startRow
    data.endRow = data.endRow
    this.setSelection(data)
  }

  _rerenderSelection() {
    var sel = this.getSelection()
    if (sel) {
      var rect = this._getRectangle(sel)
      this.refs.selection.el.css(rect)
      this.refs.selection.el.setStyle('display', 'block')
    } else {
      this._hideSelection()
    }
  }

  _hideSelection() {
    this.refs.selection.el.setStyle('display', 'none')
  }

  _activateCurrentCell(initialContent) {
    const sel = this.getSelection()
    if (sel) {
      const data = sel.data
      const row = data.startRow
      const col = data.startCol
      const cellComp = this._getCellComponentAt(row, col)
      if (cellComp) {
        this.activateCell(cellComp, initialContent)
      }
    }
  }

  _deleteSelection() {
    const sel = this.getSelection()
    if (sel) {
      const editorSession = this.getEditorSession()
      const sheet = this.getSheet()
      const data = sel.data
      const minRow = Math.min(data.startRow, data.endRow)
      const maxRow = Math.max(data.startRow, data.endRow)
      const minCol = Math.min(data.startCol, data.endCol)
      const maxCol = Math.max(data.startCol, data.endCol)
      editorSession.transaction(function(tx) {
        for (let row = minRow; row <= maxRow; row++) {
          for (let col = minCol; col <= maxCol; col++) {
            const cell = sheet.getCellAt(row, col)
            if (cell) {
              tx.set([cell.id, 'content'], '')
            }
          }
        }
      })
    }
  }

  _getCellComponentAt(row, col) {
    // HACK: using the DOM directly
    // TODO: this is a bit hacky, maybe we want to 'register' a matrix components?
    const tbody = this.refs.body.getNativeElement()
    const rowEls = tbody.children
    const rowEl = rowEls[row]
    if (rowEl) {
      const cellEls = rowEl.children
      // the first child is a th, thus col+1
      const cellEl = cellEls[col+1]
      return Component.unwrap(cellEl)
    }
  }

  _getRectangle(sel) {
    const data = sel.data
    // HACK: using the DOM directly
    const tbody = this.refs.body.getNativeElement()
    const rowEls = tbody.children
    // FIXME: due to lack of API in DOMElement
    // we are using the native API here
    const minRow = Math.min(data.startRow, data.endRow)
    const maxRow = Math.max(data.startRow, data.endRow)
    const minCol = Math.min(data.startCol, data.endCol)
    const maxCol = Math.max(data.startCol, data.endCol)

    const firstEl = rowEls[minRow].children[minCol+1]
    const lastEl = rowEls[maxRow].children[maxCol+1]

    // Now we go back into DOMElement API land
    // TODO: it would be better to have DefaultDOMElement.unwrap()
    const $firstEl = DefaultDOMElement.wrap(firstEl)
    const $lastEl = DefaultDOMElement.wrap(lastEl)
    const pos1 = $firstEl.getPosition()
    const pos2 = $lastEl.getPosition()
    const rect2 = lastEl.getBoundingClientRect()
    let rect = {
      top: pos1.top,
      left: pos1.left,
      height: pos2.top - pos1.top + rect2.height,
      width: pos2.left - pos1.left + rect2.width
    }
    return rect
  }

}
