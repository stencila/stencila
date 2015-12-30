'use strict';

var Component = require('substance/ui/Component');
var Controller = require('substance/ui/Controller');
var SheetComponent = require('./SheetComponent');
var $$ = Component.$$;

var Sheet = require('../model/Sheet');

var SheetRemoteEngine = require('../engine/SheetRemoteEngine');
var engine = new SheetRemoteEngine();

function SheetEditor() {
  SheetEditor.super.apply(this, arguments);

  this.handleActions({
    'selectedCell': this.onSelectedCell,
    'activatedCell': this.onActivatedCell,
  });

  this.selection = [0,0,0,0];
  this.startCellEl = null;
  this.endCellEl = null;

  this.onKeyDown = this.onKeyDown.bind(this);
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
    el.on('keydown', this.onKeyDown);
    return el;
  };

  this.didMount = function() {
    // ATTENTION: we need to override the hacky parent implementation

    // HACK: without contenteditables we don't receive keyboard events on this level
    window.document.body.addEventListener('keydown', this.onKeyDown, false);
    window.onresize = this.onWindowResize.bind(this);
  };

  this.dispose = function() {
    window.document.body.removeEventListener('keydown', this.onKeyDown, false);
  };

  this.onSelectedCell = function(cell) {
    if (this.activeCell && this.activeCell !== cell) {
      this.activeCell.disableEditing();
    }
    var node = cell.getNode();
    if (node) {
      console.log('Show expression bar.');
    }
    this._rerenderSelection();
    this.removeClass('edit');
  };

  this.onActivatedCell = function(cell) {
    if (this.activeCell && this.activeCell !== cell) {
      this.activeCell.disableEditing();
    }
    this.activeCell = cell;
    this._rerenderSelection();
    this.addClass('edit');
  };

  this.onMouseDown = function(event) {
    this.isSelecting = true;
    this.$el.on('mouseenter', 'td', this.onMouseEnter.bind(this));
    this.$el.one('mouseup', this.onMouseUp.bind(this));
    this.startCellEl = event.target;
    this.endCellEl = this.startCellEl;
    this._updateSelection();
  };

  this.onMouseEnter = function(event) {
    if (!this.isSelecting) return;
    if (this.endCellEl !== event.target) {
      this.endCellEl = event.target;
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

  this.onKeyDown = function(event) {
    var isEditing = this._isEditing();
    // console.log('####', event.keyCode);
    var handled = false;
    // ESCAPE
    if (event.keyCode === 27) {
      if (this.activeCell) {
        this.activeCell.disableEditing();
        this.removeClass('edit');
        this._rerenderSelection();
      }
      handled = true;
    }
    // ENTER
    else if (event.keyCode === 13) {
      if (this.activeCell) {
        var cell = this.activeCell.props.node;
        var sheet = this.refs.sheet;
        engine.update([{
          "id" : cell.cid,
          "source" : cell.expr
        }], function(error, updates){
          for(var index = 0; index < updates.length; index++){
            var update = updates[index];
            var coords = Sheet.static.getRowCol(update.id);
            var cellComponent = sheet.getCellAt(coords[0], coords[1]);
            var cellNode = cellComponent.getNode();
            cellNode.tipe = update.type;
            cellNode.value = update.value;
            cellComponent.rerender();
          }
        });
      }
      this._selectNextCell(1, 0);
      handled = true;
    }
    // TAB
    else if (event.keyCode === 9) {
      if (event.shiftKey) {
        this._selectNextCell(0, -1);
      } else {
        this._selectNextCell(0, 1);
      }
      handled = true;
    }
    // Some things are only handled if not editing, such as left right navigation
    else if (!isEditing) {
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
      // SPACE
      else if (event.keyCode === 32) {
        this._activateCurrentCell();
        handled = true;
      }
    }
    if (handled) {
      event.stopPropagation();
      event.preventDefault();
    }
  };

  this._isEditing = function() {
    return !!this.activeCell;
  };

  this._getPosition = function(el) {
    var row, col;
    if (el.hasAttribute('data-col')) {
      col = el.getAttribute('data-col');
      row = el.parentNode.getAttribute('data-row');
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
      var minRow = Math.min(startPos.row, endPos.row);
      var minCol = Math.min(startPos.col, endPos.col);
      var maxRow = Math.max(startPos.row, endPos.row);
      var maxCol = Math.max(startPos.col, endPos.col);
      var sel = [minRow, minCol, maxRow, maxCol];
      this.setSelection(sel);
    }
  };

  this._selectNextCell = function(rowDiff, colDiff) {
    var sel = this.selection;
    sel[0] = sel[0] + rowDiff;
    // TODO: also ensure upper bound
    if (rowDiff < 0) {
      sel[0] = Math.max(0, sel[0]);
    }
    sel[2] = sel[0];

    sel[1] = sel[1] + colDiff;
    // TODO: also ensure upper bound
    if (colDiff < 0) {
      sel[1] = Math.max(0, sel[1]);
    }
    sel[3] = sel[1];
    this.setSelection(sel);
  };

  this._expandSelection = function(rowDiff, colDiff) {
    var sel = this.selection;
    sel[2] = sel[2] + rowDiff;
    // TODO: also ensure upper bound
    if (rowDiff < 0) {
      sel[2] = Math.max(0, sel[2]);
    }

    sel[3] = sel[3] + colDiff;
    // TODO: also ensure upper bound
    if (colDiff < 0) {
      sel[3] = Math.max(0, sel[3]);
    }

    this.setSelection(sel);
  };

  this.setSelection = function(sel) {
    if (this.activeCell) {
      this.activeCell.disableEditing();
      this.activeCell = null;
      this.removeClass('edit');
    }
    this.selection = sel;
    this._rerenderSelection();
  };

  this._rerenderSelection = function() {
    var sel = this.selection;
    if (sel) {
      var rect = this.refs.sheet._getRectangle(sel);
      this.refs.selection.css(rect);
    }
  };

  this._activateCurrentCell = function() {
    var row = this.selection[0];
    var col = this.selection[1];
    var cell = this.refs.sheet.getCellAt(row, col);
    if (cell) {
      cell.enableEditing();
    }
  };

  this.onWindowResize = function() {
    this._rerenderSelection();
  };

};

Controller.extend(SheetEditor);

module.exports = SheetEditor;
