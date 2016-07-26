'use strict';

var Tool = require('substance/ui/Tool');

/**
 * Tool for toggling reveal mode for a Stencila Document
 * 
 * @class      RevealTool (name)
 */
function RevealTool() {
  RevealTool.super.apply(this, arguments);
}

RevealTool.Prototype = function() {
};

Tool.extend(RevealTool);

module.exports = RevealTool;

