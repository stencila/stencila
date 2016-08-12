'use strict';

var Tool = require('substance/ui/Tool');

/**
 * Tool for toggling the reveal mode of a 
 * Stencila Document `VisualEditor`
 * 
 * @class      RevealTool (name)
 */
function RevealTool() {
  RevealTool.super.apply(this, arguments);
}

RevealTool.Prototype = function() {

	this.onClick = function() {
		this.send('reveal-toggle');
	}

};

Tool.extend(RevealTool);

module.exports = RevealTool;

