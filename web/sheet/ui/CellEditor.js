'use strict';

var Component = require('substance/ui/Component');

/*
  The CellEditor is different to a regular TextPropertyEditor
  in the regard that it doesn't update the document during editing,
  only at the end.
*/
function CellEditor() {
  CellEditor.super.apply(this, arguments);
}

CellEditor.Prototype = function() {

};

module.exports = CellEditor;
