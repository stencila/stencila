'use strict';

var ControllerTool = require('substance/ui/ControllerTool');

function RenderTool() {
  RenderTool.super.apply(this, arguments);
}

ControllerTool.extend(RenderTool);

RenderTool.static.name = 'render';
RenderTool.static.command = 'render';

module.exports = RenderTool;
