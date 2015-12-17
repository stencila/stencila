'use strict';

var HTMLImporter = require('substance/model/HTMLImporter');
var DefaultDOMElement = require('substance/ui/DefaultDOMElement');
var Sheet = require('./Sheet');
var converters = require('./SheetConverters');


function SheetHTMLImporter() {
  SheetHTMLImporter.super.call(this, {
    schema: Sheet.static.schema,
    converters: converters,
    DocumentClass: Sheet,
    // TODO: this again does not make sense here, as we don't have a container
    // we should change this in DOMImporter.
    containerId: 'fixme'
  });
}

SheetHTMLImporter.Prototype = function() {

  this.importDocument = function(html) {
    // initialization
    this.reset();
    // Stencil is providing the content of body, not a full HTML document
    var table = DefaultDOMElement.parseHTML(html);

    var tbody = table.find('tbody');
    var rowEls = tbody.children;
    // ATTENTION this is a very optimistic implementation in that regard,
    // that it expects the table to be fully specified (not sparse)
    // having no spanning cells, and expects the first column to contain <th> elements only.
    for (var i = 0; i < rowEls.length; i++) {
      var rowEl = rowEls[i];
      var cellEls = rowEl.children;
      for (var j = 1; j < cellEls.length; j++) {
        var cellEl = cellEls[j];
        if (cellEl.text()) {
          var cell = this.convertElement(cellEl);
          cell.row = i;
          cell.col = j-1;
          cell.cid = Sheet.static.getCellId(cell.row, cell.col);
        }
      }
    }
    var doc = this.generateDocument();
    return doc;
  };

};

HTMLImporter.extend(SheetHTMLImporter);

module.exports = SheetHTMLImporter;
