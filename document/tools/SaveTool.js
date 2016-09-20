'use strict';

import Tool from 'substance/packages/tools/Tool'

function SaveTool () {
  SaveTool.super.apply(this, arguments);
}

SaveTool.Prototype = function () {
};

Tool.extend(SaveTool);

module.exports = SaveTool;

