'use strict';

var Tool = require('substance/ui/Tool');

function SaveTool () {

  SaveTool.super.apply(this, arguments);

}

SaveTool.Prototype = function () {
};

Tool.extend(SaveTool);

module.exports = SaveTool;

