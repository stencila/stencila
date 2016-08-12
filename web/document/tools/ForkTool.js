'use strict';

var Tool = require('substance/ui/Tool');


function ForkTool() {
  ForkTool.super.apply(this, arguments);
}

ForkTool.Prototype = function() {
};

Tool.extend(ForkTool);

module.exports = ForkTool;

