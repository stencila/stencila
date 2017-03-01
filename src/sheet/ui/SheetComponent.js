import { Component, forEach, DefaultDOMElement } from 'substance'
import Sheet from '../model/Sheet'
import TableSelection from '../model/TableSelection'
import CellComponent from './CellComponent'

export default
class SheetComponent extends Component {

  constructor(...args) {
    super(...args)

    this.handleActions({
      'commitCellChange': this.commitCellChange,
      'discardCellChange': this.discardCellChange,
      'activateCurrentCell': this._activateCurrentCell
    })

    // Shouldn't it be null rather?
    this.selection = new TableSelection({
      startRow: 0,
      startCol: 0,
      endRow: 0,
      endCol: 0
    })

    this.startCellEl = null
    this.endCellEl = null

    // binding this, as these handlers are attached to global DOM elements
    this.onGlobalKeydown = this.onGlobalKeydown.bind(this)
    this.onGlobalKeypress = this.onGlobalKeypress.bind(this)
    this.onWindowResize = this.onWindowResize.bind(this)
  }

  render($$) {
    var el = $$('div').addClass('sc-sheet-editor')
    el.append(
      this._renderTable($$)
    )
    el.append(
      $$('div').addClass('selection').ref('selection')
    )
    // react only to mousedowns on cells in display mode
    el.on('mousedown', 'td.sm-display', this.onMouseDown)
    return el
  }

  _renderTable($$) {
    // TODO: this code is almost identical to the exporter
    // we should try to share the code
    var sheet = this.props.doc

    // TODO: make this configurable
    var ncols = Math.max(52, sheet.getColumnCount())
    var nrows = Math.max(100, sheet.getRowCount())
    var tableEl = $$('table').addClass("sc-sheet")

    var i,j

    // create header row
    var thead = $$('thead')
    var headerRow = $$('tr').addClass('se-row')
    headerRow.append($$('th').addClass('se-cell'))
    for (j = 0; j < ncols; j++) {
      headerRow.append($$('th').text(
        Sheet.static.getColumnName(j)
      ).addClass('se-cell'))
    }
    thead.append(headerRow)
    tableEl.append(thead)

    var tbody = $$('tbody').ref('body')
    for (i = 0; i < nrows; i++) {
      var rowEl = $$('tr').attr('data-row', i).addClass('se-row')
      // first column is header
      rowEl.append($$('th').text(i+1).addClass('se-cell'))
      // render all cells
      for (j = 0; j < ncols; j++) {
        var cell = sheet.getCellAt(i, j)

        // Render Cell content
        var cellEl = $$(CellComponent, { node: cell })
          .attr('data-row', i)
          .attr('data-col', j)
        rowEl.append(cellEl)
      }
      tbody.append(rowEl)
    }
    tableEl.append(tbody)
    return tableEl
  }

  didMount() {
    // ATTENTION: we need to override the hacky parent implementation
    this.props.doc.connect(this, {
      'document:changed': this.onDocumentChange
    })

    // HACK: without contenteditables we don't receive keyboard events on this level
    window.document.body.addEventListener('keydown', this.onGlobalKeydown, false)
    window.document.body.addEventListener('keypress', this.onGlobalKeypress, false)
    window.addEventListener('resize', this.onWindowResize, false)
  }

  dispose() {
    this.props.doc.disconnect(this)

    window.document.body.removeEventListener('keydown', this.onGlobalKeydown)
    window.document.body.removeEventListener('keypress', this.onGlobalKeypress)
    window.removeEventListener('resize', this.onWindowResize)
  }

  getSelection() {
    return this.selection
  }

  getEditorSession() {
    return this.context.editorSession
  }

  getSheet() {
    return this.props.doc
  }

  getController() {
    return this.context.controller
  }

  setSelection(sel) {
    if (this.activeCell) {
      var cell = this.activeCell
      this.activeCell = null
      cell.commit()
      this.removeClass('sm-edit')
    }
    this.selection = new TableSelection(sel)
    if (this.props.onSelectionChanged) {
      this.props.onSelectionChanged(this.selection)
    }
    this._rerenderSelection()
  }

  // Action handlers

  selectCell(cell) {
    this._ensureActiveCellIsCommited(cell)
    this.removeClass('sm-edit')
    this._rerenderSelection()
  }

  commitCellChange(content, key) {
    if (!this.activeCell) {
      console.warn('FIXME: expected to have an active cell.')
    } else {
      var cell = this.activeCell
      this.activeCell = null
      cell.commit()
    }
    if (key === 'enter') {
      this._selectNextCell(1, 0)
    }
    this.removeClass('sm-edit')
    this._rerenderSelection()
  }

  discardCellChange() {
    var cell = this.activeCell
    this.activeCell = null
    cell.discard()
    this.removeClass('sm-edit')
    this._rerenderSelection()
  }

  // DOM event handlers

  onMouseDown(event) {
    this.isSelecting = true
    this.$el.on('mouseenter', 'td', this.onMouseEnter.bind(this))
    this.$el.one('mouseup', this.onMouseUp.bind(this))
    this.startCellEl = event.target
    if (!this.startCellEl.getAttribute('data-col')) {
      throw new Error('mousedown on a non-cell element')
    }
    this.endCellEl = this.startCellEl
    this._updateSelection()
  }

  onMouseEnter(event) {
    if (!this.isSelecting) return
    var endCellEl = this._getCellForDragTarget(event.target)
    if (this.endCellEl !== endCellEl) {
      this.endCellEl = endCellEl
      this._updateSelection()
    }
  }

  onMouseUp() {
    this.isSelecting = false
    this.$el.off('mouseenter')
    this._updateSelection()
    this.startCellEl = null
    this.endCellEl = null
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

    if (!this._isEditing()) {
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
      // SPACE
      else if (event.keyCode === 32) {
        if (this.getSelection().isCollapsed()) {
          this._toggleDisplayMode()
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

    if (!this._isEditing()) {
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

  onDocumentChange(change) {
    var cells = []
    var doc = this.props.doc
    forEach(change.created, function(nodeData) {
      if (nodeData.type === 'sheet-cell') {
        cells.push(nodeData)
      }
    })
    forEach(change.deleted, function(nodeData) {
      if (nodeData.type === 'sheet-cell') {
        cells.push(nodeData)
      }
    })
    forEach(change.updated, function(props, id) {
      var cell = doc.get(id)
      if (cell && cell.type === 'sheet-cell') {
        cells.push(cell)
      }
    })
    if (cells.length > 0) {
      this.send('updateCells', cells)
    }

    // We update the selection on each document change
    // E.g. this solves the situation where we update the display
    // mode using the DisplayMode tool where no selection update
    // is triggered.
    this._rerenderSelection()
  }

  // private API

  /**
    Sometimes we get the content elements of a cell as a target
    when we drag a selection. This method normalizes the target
    and returns always the correct cell
  */
  _getCellForDragTarget(target) {
    target = DefaultDOMElement.wrap(target)
    var targetCell
    if (target.hasClass('se-cell')) {
      targetCell = target
    } else {
      targetCell = target.findParent('.se-cell')[0]
    }
    if (!targetCell) throw Error('target cell could not be determined')
    return targetCell
  }

  _isEditing() {
    return Boolean(this.activeCell)
  }

  _ensureActiveCellIsCommited(cell) {
    if (this.activeCell && this.activeCell !== cell) {
      this.activeCell.commit()
    }
  }

  _getPosition(cellEl) {
    var row, col
    if (cellEl.hasAttribute('data-col')) {
      col = cellEl.getAttribute('data-col')
      row = cellEl.parentNode.getAttribute('data-row')
    } else {
      throw new Error('FIXME!')
    }
    return {
      row: row,
      col: col
    }
  }

  _updateSelection() {
    if (this.startCellEl) {
      var startPos = this._getPosition(this.startCellEl)
      var endPos = this._getPosition(this.endCellEl)
      var newSel = {}
      newSel.startRow = Math.min(startPos.row, endPos.row)
      newSel.startCol = Math.min(startPos.col, endPos.col)
      newSel.endRow = Math.max(startPos.row, endPos.row)
      newSel.endCol = Math.max(startPos.col, endPos.col)
      this.setSelection(newSel)
    }
  }

  _selectNextCell(rowDiff, colDiff) {
    var sel = this.getSelection().toJSON()

    sel.startRow = sel.startRow + rowDiff
    // TODO: also ensure upper bound
    if (rowDiff < 0) {
      sel.startRow = Math.max(0, sel.startRow)
    }
    sel.endRow = sel.startRow
    sel.startCol = sel.startCol + colDiff
    // TODO: also ensure upper bound
    if (colDiff < 0) {
      sel.startCol = Math.max(0, sel.startCol)
    }
    sel.endCol = sel.startCol
    this.setSelection(sel)
  }

  _expandSelection(rowDiff, colDiff) {
    var sel = this.getSelection().toJSON()

    sel.endRow = sel.endRow + rowDiff
    // TODO: also ensure upper bound
    if (rowDiff < 0) {
      sel.endRow = Math.max(0, sel.endRow)
    }

    sel.endCol = sel.endCol + colDiff
    // TODO: also ensure upper bound
    if (colDiff < 0) {
      sel.endCol = Math.max(0, sel.endCol)
    }

    sel.startRow = sel.startRow
    sel.endRow = sel.endRow
    this.setSelection(sel)
  }

  _rerenderSelection() {
    var sel = this.getSelection()
    if (sel) {
      var rect = this._getRectangle(sel)
      this.refs.selection.css(rect)
    }
  }

  _toggleDisplayMode() {
    var sel = this.getSelection()
    var row = sel.startRow
    var col = sel.startCol
    var cellComp = this._getCellComponentAt(row, col)
    if (cellComp) {
      cellComp.toggleDisplayMode()
      cellComp.rerender()
    }

    // HACK: we need to emit an onSelectionChanged event so
    // the DisplayModeTool reflects the updated displayMode
    if (this.props.onSelectionChanged) {
      this.props.onSelectionChanged(sel)
    }
    this._rerenderSelection()
  }

  _activateCurrentCell(initialContent) {
    var sel = this.getSelection()
    var row = sel.startRow
    var col = sel.startCol
    var cellComp = this._getCellComponentAt(row, col)
    if (cellComp) {
      this._ensureActiveCellIsCommited(cellComp)
      this.activeCell = cellComp
      this.addClass('sm-edit')
      cellComp.enableEditing(initialContent)
      this._rerenderSelection()
    }
  }

  _deleteSelection() {
    var sel = this.getSelection()
    var minRow = Math.min(sel.startRow, sel.endRow)
    var maxRow = Math.max(sel.startRow, sel.endRow)
    var minCol = Math.min(sel.startCol, sel.endCol)
    var maxCol = Math.max(sel.startCol, sel.endCol)
    var editorSession = this.editorSession()
    var sheet = this.getSheet()
    editorSession.transaction(function(tx) {
      for (var row = minRow; row <= maxRow; row++) {
        for (var col = minCol; col <= maxCol; col++) {
          var cell = sheet.getCellAt(row, col)
          if (cell) {
            tx.set([cell.id, 'content'], '')
          }
        }
      }
    })
    this._rerenderSelection()
  }

  _getCellComponentAt(row, col) {
    var rows = this.refs.body.children
    var rowComp = rows[row]
    if (rowComp) {
      var cells = rowComp.children
      return cells[col+1]
    }
  }

  _getRectangle(sel) {
    var rows = this.refs.body.children
    // FIXME: due to lack of API in DOMElement
    // we are using the native API here
    var minRow = Math.min(sel.startRow, sel.endRow)
    var maxRow = Math.max(sel.startRow, sel.endRow)
    var minCol = Math.min(sel.startCol, sel.endCol)
    var maxCol = Math.max(sel.startCol, sel.endCol)

    var firstEl = rows[minRow].el.childNodes[minCol+1]
    var lastEl = rows[maxRow].el.childNodes[maxCol+1]
    // debugger
    var $firstEl = DefaultDOMElement.wrap(firstEl)
    var $lastEl = DefaultDOMElement(lastEl)
    var pos1 = $firstEl.getPosition()
    var pos2 = $lastEl.getPosition()
    var rect2 = lastEl.getBoundingClientRect()
    var rect = {
      top: pos1.top,
      left: pos1.left,
      height: pos2.top - pos1.top + rect2.height,
      width: pos2.left - pos1.left + rect2.width
    }
    return rect
  }

}
