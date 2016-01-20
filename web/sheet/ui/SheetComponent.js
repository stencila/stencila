"use strict";

var Component = require('substance/ui/Component');
var CellComponent = require('./CellComponent');
var Sheet = require('../model/Sheet');
var $ = require('substance/util/jquery');
var $$ = Component.$$;

function SheetComponent() {
  SheetComponent.super.apply(this, arguments);
}

SheetComponent.Prototype = function() {

  this.render = function() {
    // TODO this code is almost identical to the exporter
    // we should try to share the code
    var doc = this.props.doc;
    var tableData = doc.getTableData('all');
    // always render a certain
    // TODO: make this configurable
    var ncols = Math.max(52, tableData.cols);
    var nrows = Math.max(100, tableData.rows);
    var table = $$('table')
      .addClass("sc-sheet");
    var i,j;

    // create header row
    var thead = $$('thead');
    var headerRow = $$('tr').addClass('se-row');
    headerRow.append($$('th').addClass('se-cell'));
    for (j = 0; j < ncols; j++) {
      headerRow.append($$('th').text(Sheet.static.getColumnName(j)).addClass('se-cell'));
    }
    thead.append(headerRow);
    table.append(thead);

    var tbody = $$('tbody').ref('body');
    for (i = 0; i < nrows; i++) {
      var rowEl = $$('tr').attr('data-row', i).addClass('se-row');
      // first column is header
      rowEl.append($$('th').text(i+1).addClass('se-cell'));
      // render all cells
      for (j = 0; j < ncols; j++) {
        var cell = tableData.cells[[i,j]];
        var cellEl = $$(CellComponent, { node: cell })
          .attr('data-row', i)
          .attr('data-col', j);
        rowEl.append(cellEl);
      }
      tbody.append(rowEl);
    }
    table.append(tbody);

    return table;
  };

  this._getRectangle = function(sel) {
    var rows = this.refs.body.children;
    // FIXME: due to lack of API in DOMElement
    // we are using the native API here
    var minRow = Math.min(sel[0], sel[2]);
    var maxRow = Math.max(sel[0], sel[2]);
    var minCol = Math.min(sel[1], sel[3]);
    var maxCol = Math.max(sel[1], sel[3]);
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

  this.getCellAt = function(row, col) {
    var rows = this.refs.body.children;
    var rowComp = rows[row];
    if (rowComp) {
      var cells = rowComp.children;
      return cells[col+1];
    }
  };

};

Component.extend(SheetComponent);

module.exports = SheetComponent;