import {
  SplitPane, ScrollPane,
  // Toolbar,
  AbstractEditor
} from 'substance'

// import DisplayModeTool from './ui/DisplayModeTool'
import SheetComponent from './ui/SheetComponent'
import SheetModel from './model/SheetModel'

export default
class SheetEditor extends AbstractEditor {

  constructor(...args) {
    super(...args)

    this.handleActions({
      'updateCells': this.updateCells
    })
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-editor').append(
      this.renderMainSection($$)
    )
    return el
  }

  renderMainSection($$) {
    // FIXME: This was the old cold
    // return $$('div').ref('main').addClass('se-main-section').append(
    //   $$(SplitPane, {splitType: 'horizontal'}).append(
    //     // Menu Pane on top
    //     $$(Toolbar).ref('toolbar').append(
    //       $$(Toolbar.Group).append(
    //         // $$(HomeTool, {
    //         //   address: this.props.engine.address
    //         // }).ref('homeTool')
    //       ),
    //       $$(Toolbar.Group).addClass('float-right').append(
    //         // $$(UndoTool).append($$(Icon, {icon: 'fa-undo'})),
    //         // $$(RedoTool).append($$(Icon, {icon: 'fa-repeat'})),
    //         // Removed for now, see #132
    //         //$$(SaveTool).append($$(Icon, {icon: 'fa-save'})),
    //         //$$(CommitTool)
    //         $$(DisplayModeTool).ref('displayModeTool')
    //       )
    //     ),
    //     // Content Panel below
    //     $$(ScrollPane, {
    //       scrollbarPosition: 'left'
    //     }).ref('contentPanel').append(
    //       $$('div').ref('main').addClass('document-content').append(
    //         $$(SheetComponent, {
    //           mode: this.props.mode,
    //           doc: this.props.doc,
    //           onSelectionChanged: this._onSelectionChanged.bind(this)
    //         }).ref('sheetEditor')
    //       )
    //     )
    //   ).ref('mainSectionSplitPane')
    // )
    return $$('div').ref('main').addClass('se-main-section').append(
      $$(ScrollPane, {
        scrollbarPosition: 'left'
      }).ref('contentPanel').append(
        $$('div').ref('content').addClass('document-content').append(
          $$(SheetComponent, {
            mode: this.props.mode,
            doc: this.props.editorSession.getDocument(),
            onSelectionChanged: this._onSelectionChanged.bind(this)
          }).ref('sheetEditor')
        )
      )
    )
  }

  updateCells(cells) {
    cells = cells.map(function(cell) {
      return {
        id: SheetModel.getCellId(cell.row, cell.col),
        source: cell.content || '',
        display: cell.displayMode
      }
    })
    // Update the sheet with the new cell source
    this.props.engine.update(cells, function(err, updates) {
      if (err) {
        this.getLogger().error(err.message || err.toString())
        return
      }
      if (!updates) {
        console.error('FIXME: did not receive updates.', updates)
        return
      }
      this._handleUpdates(updates)
    }.bind(this))
  }

  _handleUpdates(updates) {
    var sheet = this.props.doc
    for(var index = 0; index < updates.length; index++) {
      var update = updates[index]
      var coords = SheetModel.getRowCol(update.id)
      var cell = sheet.getCellAt(coords[0], coords[1])
      if (cell) {
        cell.kind = update.kind
        cell.valueType = update.type
        cell.value = update.value
        cell.displayMode = update.display
        cell.emit('cell:changed')
      }
    }
    if (updates.length > 0) {
      this.refs.sheetEditor._rerenderSelection()
    }
  }

  /*
    Called when the table selection is changed so we can
    update the display mode tool accordingly.
  */
  _onSelectionChanged(sel) {
    var displayModeTool = this.refs.displayModeTool
    var doc = this.props.doc
    var cell
    if (sel.isCollapsed) {
      cell = doc.getCellAt(sel.startRow, sel.startCol)
    }
    if (cell) {
      displayModeTool.setState({
        disabled: false,
        cell: cell
      })
      return
    } else {
      displayModeTool.setState({
        disabled: true,
        cell: null
      })
    }
  }

}
