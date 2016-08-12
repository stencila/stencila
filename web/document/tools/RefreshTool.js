'use strict';

var Tool = require('substance/ui/Tool');


function RefreshTool() {
  RefreshTool.super.apply(this, arguments);
}

RefreshTool.Prototype = function() {
};

Tool.extend(RefreshTool);

module.exports = RefreshTool;

