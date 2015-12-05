'use strict';

var Component = require('substance/ui/Component');
var CellComponent = require('./CellComponent');
var $$ = Component.$$;

function SheetComponent() {
  SheetComponent.super.apply(this, arguments);
}

SheetComponent.Prototype = function() {

  this.render = function() {
    var doc = this.props.doc;
    var tableData = doc.getTableData();
    // always render a certain
    // TODO: make this configurable
    var ncols = Math.max(52, tableData.cols);
    var nrows = Math.max(100, tableData.rows);
    var table = $$('table')
      .addClass("stencila-sheet");
    for (var i = 0; i < nrows; i++) {
      var row = $$('tr').attr('data-row', i);
      for (var j = 0; j < ncols; j++) {
        var cellNode = tableData.cells[[i,j]];
        row.append($$(CellComponent, { node: cellNode }));
      }
      table.append(row);
    }
    return table;
  };

};

Component.extend(SheetComponent);

module.exports = SheetComponent;
