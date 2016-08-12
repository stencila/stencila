'use strict';

var Tool = require('substance/ui/Tool');

/**
 * Tool for toggling edit mode for a Stencila Document
 * `VisualEditor`
 * 
 * @class      EditTool (name)
 */
function EditTool() {
  EditTool.super.apply(this, arguments);
}

EditTool.Prototype = function() {

	this.onClick = function() {
		this.send('edit-toggle');
	}

};

Tool.extend(EditTool);

module.exports = EditTool;

