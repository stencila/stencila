'use strict';

var HTMLExporter = require('substance/model/HTMLExporter');
var Sheet = require('./Sheet');
var converters = require('./SheetConverters');

function SheetHTMLExporter() {
  SheetHTMLExporter.super.call(this, {
    converters: converters
  });
}

SheetHTMLExporter.Prototype = function() {
  this.exportDocument = function(sheet) {
    var $$ = this.$$;
    var ncols = sheet.getColumnCount();
    var nrows = sheet.getRowCount();
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
        var cell = sheet.getCellAt(i, j);
        var cellEl = this.convertCell(cell);
        rowEl.append(cellEl);
      }
      tbody.append(rowEl);
    }
    tableEl.append(tbody);
    return tableEl.outerHTML;
  };

  this.convertCell = function(cell) {
    var $$ = this.$$;
    if (!cell) {
      return $$('td');
    } else {
      return this.convertNode(cell);
    }
  };

};

HTMLExporter.extend(SheetHTMLExporter);

module.exports = SheetHTMLExporter;