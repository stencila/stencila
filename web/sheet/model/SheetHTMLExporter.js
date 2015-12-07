'use strict';

var HTMLExporter = require('substance/model/HTMLExporter');
var Sheet = require('./Sheet');
var converters = require('./SheetConverters');

function StencilHTMLExporter() {
  StencilHTMLExporter.super.call(this, {
    converters: converters
  });
}

StencilHTMLExporter.Prototype = function() {
  this.exportDocument = function(doc) {
    var $$ = this.$$;
    var tableData = doc.getTableData('sparse');
    // always render a certain
    // TODO: make this configurable
    var ncols = tableData.cols;
    var nrows = tableData.rows;
    var i,j;

    var tableEl = $$('table');
    // create header
    var thead = $$('thead').append('tr');
    tableEl.append(thead);
    var headerRow = thead.firstChild;
    headerRow.append($$('th'));
    for (j = 0; j < ncols; j++) {
      headerRow.append($$('th').text(Sheet.static.getColumnName(j)));
    }
    // create
    var tbody = $$('tbody');
    for (i = 0; i < nrows; i++) {
      var rowEl = $$('tr');
      rowEl.append($$('th').text(i+1));
      for (j = 0; j < ncols; j++) {
        var cell = tableData.cells[[i,j]];
        var cellEl = this.convertNode(cell);
        rowEl.append(cellEl);
      }
      tbody.append(rowEl);
    }
    tableEl.append(tbody);
    return tableEl.outerHTML;
  };

};

HTMLExporter.extend(StencilHTMLExporter);

module.exports = StencilHTMLExporter;