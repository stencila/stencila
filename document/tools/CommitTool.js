'use strict';

import Tool from 'substance/packages/tools/Tool'

function CommitTool () {
  CommitTool.super.apply(this, arguments);
}

CommitTool.Prototype = function () {
};

Tool.extend(CommitTool);

module.exports = CommitTool;

