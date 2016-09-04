'use strict';

var Tool = require('substance/ui/Tool');

function RefreshTool () {
  RefreshTool.super.apply(this, arguments);
}

RefreshTool.Prototype = function () {
  this.getTitle = function () {
    return 'Refresh computations; not yet implemented :(';
  };
};

Tool.extend(RefreshTool);

module.exports = RefreshTool;

