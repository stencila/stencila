'use strict';

var ControllerTool = require('substance/ui/ControllerTool');

function ActivateTool() {
  ActivateTool.super.apply(this, arguments);
}

ControllerTool.extend(ActivateTool);

ActivateTool.static.name = 'activate';
ActivateTool.static.command = 'activate';

module.exports = ActivateTool;
