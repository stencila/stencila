"use strict";

var SurfaceTool = require('substance/ui/SurfaceTool');

function StencilIncludeInsertTool() {
  StencilIncludeInsertTool.super.apply(this, arguments);
}

SurfaceTool.extend(StencilIncludeInsertTool);

StencilIncludeInsertTool.static.name = 'stencil-include-insert';
StencilIncludeInsertTool.static.command = 'stencil-include-insert';

module.exports = StencilIncludeInsertTool;
