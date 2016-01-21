'use strict';

var Component = require('substance/ui/Component');
var SheetComponent = require('./SheetComponent');
var TableSelection = require('../model/TableSelection');
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
      $$(SheetComponent, { doc: this.props.doc }).ref('sheet')
    );
    el.append(
      $$('div').addClass('selection').ref('selection')
    );
    // react only to mousedowns on cells in display mode
    el.on('mousedown', 'td.display', this.onMouseDown);
    return el;
  };

  this.didMount = function() {
    // ATTENTION: we need to override the hacky parent implementation

    // HACK: without contenteditables we don't receive keyboard events on this level
    window.document.body.addEventListener('keydown', this.onGlobalKeydown, false);
    window.document.body.addEventListener('keypress', this.onGlobalKeypress, false);
    window.addEventListener('resize', this.onWindowResize, false);
  };

  this.dispose = function() {
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

  this.getController = function() {
    return this.context.controller;
  };

  // Action handlers

  this.selectCell = function(cell) {
    this._ensureActiveCellIsCommited(cell);
    this.removeClass('edit');
    this._rerenderSelection();
  };

  this.activateCell = function(cell) {
    this._ensureActiveCellIsCommited(cell);
    this.activeCell = cell;
    this.addClass('edit');
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
    this.removeClass('edit');
    this._rerenderSelection();
  };

  this.discardCellChange = function() {
    var cell = this.activeCell;
    this.activeCell = null;
    cell.disableEditing();
    this.removeClass('edit');
    this._rerenderSelection();
  };

  // DOM event handlers

  this.onMouseDown = function(event) {
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
    }

    if (handled) {
      console.log('SheetEditor.onGlobalKeydown() handled event', event);
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
      // ENTER
      if (event.keyCode === 13) {
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
      } else {
        var character = String.fromCharCode(event.charCode);
        if (character) {
          console.log('TODO: overwrite cell content and activate cell editing.');
          handled = true;
        }
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
      var sheet = this.refs.sheet;
      this.getDocumentSession().transaction(function(tx) {
        tx.set([cellNode.id, 'content'], content);
      });
      this.send('updateCell', cellNode, sheet);
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

  this.setSelection = function(sel) {
    if (this.activeCell) {
      this.activeCell.disableEditing();
      this.activeCell = null;
      this.removeClass('edit');
    }
    this.selection = new TableSelection(sel);

    // Reset
    // if (this.selectedCell) {
    //   this.selectedCell.extendProps({
    //     selected: false
    //   });
    // }
    // if (this.selection.isCollapsed()) {
    //   this.selectedCell = this.refs.sheet.getCellAt(sel.startRow, sel.startCol);
    //   this.selectedCell.extendProps({
    //     selected: true
    //   });
    // }
    this._rerenderSelection();
  };

  this._rerenderSelection = function() {
    var sel = this.getSelection();
    if (sel) {
      var rect = this.refs.sheet._getRectangle(sel);
      this.refs.selection.css(rect);
    }
  };

  this._toggleDisplayMode = function() {
    var sel = this.getSelection();
    var row = sel.startRow;
    var col = sel.startCol;
    var cell = this.refs.sheet.getCellAt(row, col);
    if (cell) {
      cell.toggleDisplayMode();
      cell.rerender();
    }
    this._rerenderSelection();
  };

  this._activateCurrentCell = function() {
    var sel = this.getSelection();
    var row = sel.startRow;
    var col = sel.startCol;
    var cell = this.refs.sheet.getCellAt(row, col);
    if (cell) {
      cell.enableEditing();
    }
  };

  this._deleteSelection = function() {
    var sel = this.getSelection();
    var minRow = Math.min(sel.startRow, sel.endRow);
    var maxRow = Math.max(sel.startRow, sel.endRow);
    var minCol = Math.min(sel.startCol, sel.endCol);
    var maxCol = Math.max(sel.startCol, sel.endCol);
    var docSession = this.getDocumentSession();
    docSession.transaction(function(tx) {
      for (var row = minRow; row <= maxRow; row++) {
        for (var col = minCol; col <= maxCol; col++) {
          var cellComp = this.refs.sheet.getCellAt(row, col);
          var cell = cellComp.props.node;
          if (cell) {
            tx.set([cell.id, 'content'], '');
          }
        }
      }
    }.bind(this));
  };

};

Component.extend(SheetEditor);

module.exports = SheetEditor;
