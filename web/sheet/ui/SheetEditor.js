'use strict';

var each = require('lodash/collection/each');
var Sheet = require('../model/Sheet');
var TableSelection = require('../model/TableSelection');
var Component = require('substance/ui/Component');

var CellComponent = require('./CellComponent');
var ExpressionCell = require('./ExpressionCell');
var ConstantCell = require('./ConstantCell');

var $$ = Component.$$;
var $ = require('substance/util/jquery');

function SheetEditor() {
  SheetEditor.super.apply(this, arguments);

  this.handleActions({
    'selectCell': this.selectCell,
    'activateCell': this.activateCell,
    'commitCellChange': this.commitCellChange,
    'discardCellChange': this.discardCellChange,
  });

  // Shouldn't it be null rather?
  this.selection = new TableSelection({
    startRow: 0,
    startCol: 0,
    endRow: 0,
    endCol: 0
  });

  this.startCellEl = null;
  this.endCellEl = null;

  // binding this, as these handlers are attached to global DOM elements
  this.onGlobalKeydown = this.onGlobalKeydown.bind(this);
  this.onGlobalKeypress = this.onGlobalKeypress.bind(this);
  this.onWindowResize = this.onWindowResize.bind(this);
}

SheetEditor.Prototype = function() {

  this.render = function() {
    var el = $$('div').addClass('sc-sheet-editor');
    el.append(
      this._renderTable()
    );
    el.append(
      $$('div').addClass('selection').ref('selection')
    );
    // react only to mousedowns on cells in display mode
    el.on('mousedown', 'td.sm-display', this.onMouseDown);
    return el;
  };

  this._renderTable = function() {
    // TODO: this code is almost identical to the exporter
    // we should try to share the code
    var sheet = this.props.doc;
    var componentRegistry = this.context.componentRegistry;

    // TODO: make this configurable
    var ncols = Math.max(52, sheet.getColumnCount());
    var nrows = Math.max(100, sheet.getRowCount());
    var tableEl = $$('table').addClass("sc-sheet");

    var i,j;

    // create header row
    var thead = $$('thead');
    var headerRow = $$('tr').addClass('se-row');
    headerRow.append($$('th').addClass('se-cell'));
    for (j = 0; j < ncols; j++) {
      headerRow.append($$('th').text(
        Sheet.static.getColumnName(j)
      ).addClass('se-cell'));
    }
    thead.append(headerRow);
    tableEl.append(thead);

    var tbody = $$('tbody').ref('body');
    for (i = 0; i < nrows; i++) {
      var rowEl = $$('tr').attr('data-row', i).addClass('se-row');
      // first column is header
      rowEl.append($$('th').text(i+1).addClass('se-cell'));
      // render all cells
      for (j = 0; j < ncols; j++) {
        var cell = sheet.getCellAt(i, j);

        // Render Cell content
        var CellComponentClass;

        if (cell) {
          if (cell.isConstant()) {
            CellComponentClass = ConstantCell;
          } else if (cell.valueType) {
            CellComponentClass = componentRegistry.get(cell.valueType);
          } 

          if (!CellComponentClass) {
            CellComponentClass = ExpressionCell;
          }
        } else {
          CellComponentClass = CellComponent;
        }

        var cellEl = $$(CellComponentClass, { node: cell })
          .attr('data-row', i)
          .attr('data-col', j);
        rowEl.append(cellEl);
      }
      tbody.append(rowEl);
    }
    tableEl.append(tbody);

    return tableEl;
  };

  this.didMount = function() {
    // ATTENTION: we need to override the hacky parent implementation
    this.props.doc.connect(this, {
      'document:changed': this.onDocumentChange
    });

    // HACK: without contenteditables we don't receive keyboard events on this level
    window.document.body.addEventListener('keydown', this.onGlobalKeydown, false);
    window.document.body.addEventListener('keypress', this.onGlobalKeypress, false);
    window.addEventListener('resize', this.onWindowResize, false);
  };

  this.dispose = function() {
    this.props.doc.disconnect(this);

    window.document.body.removeEventListener('keydown', this.onGlobalKeydown);
    window.document.body.removeEventListener('keypress', this.onGlobalKeypress);
    window.removeEventListener('resize', this.onWindowResize);
  };

  this.getSelection = function() {
    return this.selection;
  };

  this.getDocumentSession = function() {
    return this.context.documentSession;
  };

  this.getSheet = function() {
    return this.props.doc;
  };

  this.getController = function() {
    return this.context.controller;
  };

  this.setSelection = function(sel) {
    if (this.activeCell) {
      this.activeCell.disableEditing();
      this.activeCell = null;
      this.removeClass('sm-edit');
    }
    this.selection = new TableSelection(sel);

    this._rerenderSelection();
  };

  // Action handlers

  this.selectCell = function(cell) {
    this._ensureActiveCellIsCommited(cell);
    this.removeClass('sm-edit');
    this._rerenderSelection();
  };

  this.activateCell = function(cell) {
    this._ensureActiveCellIsCommited(cell);
    this.activeCell = cell;
    this.addClass('sm-edit');
    this._rerenderSelection();
  };

  this.commitCellChange = function(content, key) {
    if (!this.activeCell) {
      console.warn('FIXME: expected to have an active cell.');
    } else {
      var cell = this.activeCell;
      this.activeCell = null;
      this._commitCellContent(cell, content);
      cell.disableEditing();
    }
    if (key === 'enter') {
      this._selectNextCell(1, 0);
    }
    this.removeClass('sm-edit');
    this._rerenderSelection();
  };

  this.discardCellChange = function() {
    var cell = this.activeCell;
    this.activeCell = null;
    cell.disableEditing();
    this.removeClass('sm-edit');
    this._rerenderSelection();
  };

  // DOM event handlers

  this.onMouseDown = function(event) {
    console.log('mouse down');
    this.isSelecting = true;
    this.$el.on('mouseenter', 'td', this.onMouseEnter.bind(this));
    this.$el.one('mouseup', this.onMouseUp.bind(this));
    this.startCellEl = event.target;
    if (!this.startCellEl.getAttribute('data-col')) {
      throw new Error('mousedown on a non-cell element');
    }
    this.endCellEl = this.startCellEl;
    this._updateSelection();
  };

  this.onMouseEnter = function(event) {
    if (!this.isSelecting) return;
    var endCellEl = this._getCellForDragTarget(event.target);
    if (this.endCellEl !== endCellEl) {
      this.endCellEl = endCellEl;
      this._updateSelection();
    }
  };

  this.onMouseUp = function() {
    this.isSelecting = false;
    this.$el.off('mouseenter');
    this._updateSelection();
    this.startCellEl = null;
    this.endCellEl = null;
  };

  /*
    Will be bound to body element to receive events while not
    editing a cell.
    Note: these need to be done on keydown to prevent default browser
    behavior.
  */
  this.onGlobalKeydown = function(event) {
    // console.log('onGlobalKeydown()', 'keyCode=', event.keyCode);
    var handled = false;

    if (!this._isEditing()) {
      // LEFT
      if (event.keyCode === 37) {
        if (event.shiftKey) {
          this._expandSelection(0, -1);
        } else {
          this._selectNextCell(0, -1);
        }
        handled = true;
      }
      // RIGHT
      else if (event.keyCode === 39) {
        if (event.shiftKey) {
          this._expandSelection(0, 1);
        } else {
          this._selectNextCell(0, 1);
        }
        handled = true;
      }
      // UP
      else if (event.keyCode === 38) {
        if (event.shiftKey) {
          this._expandSelection(-1, 0);
        } else {
          this._selectNextCell(-1, 0);
        }
        handled = true;
      }
      // DOWN
      else if (event.keyCode === 40) {
        if (event.shiftKey) {
          this._expandSelection(1, 0);
        } else {
          this._selectNextCell(1, 0);
        }
        handled = true;
      }
      // ENTER
      else if (event.keyCode === 13) {
        if (this.getSelection().isCollapsed()) {
          this._activateCurrentCell();
        }
        handled = true;
      }
      // SPACE
      else if (event.keyCode === 32) {
        if (this.getSelection().isCollapsed()) {
          this._toggleDisplayMode();
        }
        handled = true;
      }
      // BACKSPACE | DELETE
      else if (event.keyCode === 8 || event.keyCode === 46) {
        console.log('DELETING');
        this._deleteSelection();
        handled = true;
      }
      // undo/redo
      else if (event.keyCode === 90 && (event.metaKey||event.ctrlKey)) {
        if (event.shiftKey) {
          this.getController().executeCommand('redo');
        } else {
          this.getController().executeCommand('undo');
        }
        handled = true;
      }
    }

    if (handled) {
      // console.log('SheetEditor.onGlobalKeydown() handled event', event);
      event.stopPropagation();
      event.preventDefault();
    }
  };

  /*
    Will be bound to body element to receive events while not
    editing a cell.
    Note: only 'keypress' allows us to detect key events which
    would result in content changes.
  */
  this.onGlobalKeypress = function(event) {
    // console.log('onGlobalKeypress()', 'keyCode=', event.keyCode);
    var handled = false;

    if (!this._isEditing()) {
      var character = String.fromCharCode(event.charCode);
      if (character) {
        console.log('TODO: overwrite cell content and activate cell editing.');
        handled = true;
      }
    }

    if (handled) {
      event.stopPropagation();
      event.preventDefault();
    }
  };

  this.onWindowResize = function() {
    this._rerenderSelection();
  };

  this.onDocumentChange = function(change) {
    var cells = [];

    var doc = this.props.doc;
    each(change.created, function(nodeData) {
      // HACK this does not work currently
      // if (nodeData.type === 'sheet-cell') {
      //   cells.push(nodeData);
      // }
    });
    each(change.deleted, function(nodeData) {
      if (nodeData.type === 'sheet-cell') {
        cells.push(nodeData);
      }
    });
    each(change.updated, function(props, id) {
      var cell = doc.get(id);
      if (cell && cell.type === 'sheet-cell') {
        cells.push(cell);
      }
    });

    if (cells.length > 0) {
      this.send('updateCells', cells);
    }
  };

  // private API

  /**
    Sometimes we get the content elements of a cell as a target
    when we drag a selection. This method normalizes the target
    and returns always the correct cell
  */
  this._getCellForDragTarget = function(target) {
    var targetCell;
    if ($(target).hasClass('se-cell')) {
      targetCell = target;
    } else {
      targetCell = $(target).parents('.se-cell')[0];
    }
    if (!targetCell) throw Error('target cell could not be determined');
    return targetCell;
  };

  this._isEditing = function() {
    return !!this.activeCell;
  };

  this._commitCellContent = function(cell, content) {
    var cellNode = cell.props.node;
    if (cellNode.content !== content) {
      this.getDocumentSession().transaction(function(tx) {
        tx.set([cellNode.id, 'content'], content);
      });
    }
  };

  this._ensureActiveCellIsCommited = function(cell) {
    if (this.activeCell && this.activeCell !== cell) {
      this._commitCellContent(this.activeCell,
        this.activeCell.getCellEditorContent());
      this.activeCell.disableEditing();
    }
  };

  this._getPosition = function(cellEl) {
    var row, col;
    if (cellEl.hasAttribute('data-col')) {
      col = cellEl.getAttribute('data-col');
      row = cellEl.parentNode.getAttribute('data-row');
    } else {
      throw new Error('FIXME!');
    }
    return {
      row: row,
      col: col
    };
  };

  this._updateSelection = function() {
    if (this.startCellEl) {
      var startPos = this._getPosition(this.startCellEl);
      var endPos = this._getPosition(this.endCellEl);
      var newSel = {};
      newSel.startRow = Math.min(startPos.row, endPos.row);
      newSel.startCol = Math.min(startPos.col, endPos.col);
      newSel.endRow = Math.max(startPos.row, endPos.row);
      newSel.endCol = Math.max(startPos.col, endPos.col);
      this.setSelection(newSel);
    }
  };

  this._selectNextCell = function(rowDiff, colDiff) {
    var sel = this.getSelection().toJSON();

    sel.startRow = sel.startRow + rowDiff;
    // TODO: also ensure upper bound
    if (rowDiff < 0) {
      sel.startRow = Math.max(0, sel.startRow);
    }
    sel.endRow = sel.startRow;
    sel.startCol = sel.startCol + colDiff;
    // TODO: also ensure upper bound
    if (colDiff < 0) {
      sel.startCol = Math.max(0, sel.startCol);
    }
    sel.endCol = sel.startCol;
    this.setSelection(sel);
  };

  this._expandSelection = function(rowDiff, colDiff) {
    var sel = this.getSelection().toJSON();

    sel.endRow = sel.endRow + rowDiff;
    // TODO: also ensure upper bound
    if (rowDiff < 0) {
      sel.endRow = Math.max(0, sel.endRow);
    }

    sel.endCol = sel.endCol + colDiff;
    // TODO: also ensure upper bound
    if (colDiff < 0) {
      sel.endCol = Math.max(0, sel.endCol);
    }

    sel.startRow = sel.startRow;
    sel.endRow = sel.endRow;
    this.setSelection(sel);
  };

  this._rerenderSelection = function() {
    var sel = this.getSelection();
    if (sel) {
      var rect = this._getRectangle(sel);
      this.refs.selection.css(rect);
    }
  };

  this._toggleDisplayMode = function() {
    var sel = this.getSelection();
    var row = sel.startRow;
    var col = sel.startCol;
    var cellComp = this._getCellComponentAt(row, col);
    if (cellComp) {
      cellComp.toggleDisplayMode();
      cellComp.rerender();
    }
    this._rerenderSelection();
  };

  this._activateCurrentCell = function() {
    var sel = this.getSelection();
    var row = sel.startRow;
    var col = sel.startCol;
    var cellComp = this._getCellComponentAt(row, col);
    if (cellComp) {
      cellComp.enableEditing();
    }
  };

  this._deleteSelection = function() {
    var sel = this.getSelection();
    var minRow = Math.min(sel.startRow, sel.endRow);
    var maxRow = Math.max(sel.startRow, sel.endRow);
    var minCol = Math.min(sel.startCol, sel.endCol);
    var maxCol = Math.max(sel.startCol, sel.endCol);
    var docSession = this.getDocumentSession();
    var sheet = this.getSheet();
    docSession.transaction(function(tx) {
      for (var row = minRow; row <= maxRow; row++) {
        for (var col = minCol; col <= maxCol; col++) {
          var cell = sheet.getCellAt(row, col);
          if (cell) {
            tx.set([cell.id, 'content'], '');
          }
        }
      }
    }.bind(this));
  };

  this._getCellComponentAt = function(row, col) {
    var rows = this.refs.body.children;
    var rowComp = rows[row];
    if (rowComp) {
      var cells = rowComp.children;
      return cells[col+1];
    }
  };

  this._getRectangle = function(sel) {
    var rows = this.refs.body.children;
    // FIXME: due to lack of API in DOMElement
    // we are using the native API here
    var minRow = Math.min(sel.startRow, sel.endRow);
    var maxRow = Math.max(sel.startRow, sel.endRow);
    var minCol = Math.min(sel.startCol, sel.endCol);
    var maxCol = Math.max(sel.startCol, sel.endCol);

    var firstEl = rows[minRow].el.childNodes[minCol+1];
    var lastEl = rows[maxRow].el.childNodes[maxCol+1];
    // debugger;
    var $firstEl = $(firstEl);
    var $lastEl = $(lastEl);
    var pos1 = $firstEl.position();
    var pos2 = $lastEl.position();
    var rect2 = lastEl.getBoundingClientRect();
    var rect = {
      top: pos1.top,
      left: pos1.left,
      height: pos2.top - pos1.top + rect2.height,
      width: pos2.left - pos1.left + rect2.width
    };
    return rect;
  };

};

Component.extend(SheetEditor);

module.exports = SheetEditor;
