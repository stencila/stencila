'use strict';

var Tool = require('substance/ui/Tool');

/**
 * Tool for toggling edit mode for a Stencila Document
 * 
 * @class      EditTool (name)
 */
function EditTool() {
  EditTool.super.apply(this, arguments);
}

EditTool.Prototype = function() {
};

Tool.extend(EditTool);

module.exports = EditTool;

