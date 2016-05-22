"use strict";

var SurfaceTool = require('substance/ui/SurfaceTool');

function StencilExecInsertTool() {
  StencilExecInsertTool.super.apply(this, arguments);
}

SurfaceTool.extend(StencilExecInsertTool);

StencilExecInsertTool.static.name = 'stencil-exec-insert';
StencilExecInsertTool.static.command = 'stencil-exec-insert';

module.exports = StencilExecInsertTool;
