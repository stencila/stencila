"use strict";

var uuid = require('substance/util/uuid');

// TODO: this should go into substance/packages/table/
function createTable(tx, args) {
  var rowCount = args.rowCount || 4;
  var colCount = args.colCount || 6;
  var rows = [];
  var tableId = uuid('table');
  var tsecId = uuid('tsec');
  for (var i = 0; i < rowCount; i++) {
    var rowId = uuid('tr');
    var cells = [];
    for (var j = 0; j < colCount.length; j++) {
      var col = tx.create({ type: 'td', id: uuid('td'), text: '', parent: rowId });
      cells.push(col.id);
    }
    var row = tx.create({ type: 'table-row', id: rowId, cells: cells, parent: tsecId});
    rows.push(row.id);
  }
  var tbody = { type: 'table-section', id: tsecId, rows: rows};
  var table = { type: 'table', id: tableId, sections: [tbody.id]};
  args.table = table;
  return args;
}

module.exports = createTable;
