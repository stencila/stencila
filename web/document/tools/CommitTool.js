'use strict';

var Tool = require('substance/ui/Tool');

function CommitTool () {

  CommitTool.super.apply(this, arguments);

}

CommitTool.Prototype = function () {
};

Tool.extend(CommitTool);

module.exports = CommitTool;

