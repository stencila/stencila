'use strict';

var Component = require('substance/ui/Component');
var Controller = require('substance/ui/Controller');
var CellComponent = require('./CellComponent');
var Sheet = require('../model/Sheet');
var $$ = Component.$$;

function SheetEditor() {
  SheetEditor.super.apply(this, arguments);

  this.handleActions({
    'selectCell': this.onSelectCell,
    'activateCell': this.onActivateCell,
  });
}

SheetEditor.Prototype = function() {

  this.didMount = function() {
    // HACK: to override the hacky parent implementation
  };

  this.render = function() {
    // TODO this code is almost identical to the exporter
    // we should try to share the code

    var doc = this.props.doc;
    window._sheet = doc;
    var tableData = doc.getTableData();
    // always render a certain
    // TODO: make this configurable
    var ncols = Math.max(52, tableData.cols);
    var nrows = Math.max(100, tableData.rows);
    var el = $$('table')
      .addClass("stencila-sheet");

    var i,j;

    // create header row
    var thead = $$('thead');
    var headerRow = $$('tr');
    headerRow.append($$('th'));
    for (j = 0; j < ncols; j++) {
      headerRow.append($$('th').text(Sheet.static.getColumnName(j)));
    }
    thead.append(headerRow);
    el.append(thead);
    //
    var tbody = $$('tbody');
    for (i = 0; i < nrows; i++) {
      var rowEl = $$('tr').attr('data-row', i);
      // first column is header
      rowEl.append($$('th').text(i+1));
      // render all cells
      for (j = 0; j < ncols; j++) {
        var cell = tableData.cells[[i,j]];
        var cellEl = $$(CellComponent, { node: cell }).attr('data-col', j-1);
        rowEl.append(cellEl);
      }
      tbody.append(rowEl);
    }
    el.append(tbody);
    return el;
  };

  this.onSelectCell = function(cell) {
    if (this.activeCell && this.activeCell !== cell) {
      this.activeCell.setState({ isEditing: false });
    }
    var node = cell.getNode();
    if (node && node.isExpression()) {
      console.log('Show expression bar.');
    }
  };

  this.onActivateCell = function(cell) {
    if (this.activeCell && this.activeCell !== cell) {
      this.activeCell.setState({ isEditing: false });
    }
    this.activeCell = cell;
    cell.setState({ isEditing: true });
    cell.initializeSelection();
  };

};

Controller.extend(SheetEditor);

module.exports = SheetEditor;
