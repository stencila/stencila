"use strict";

var SurfaceTool = require('substance-fe0ed/ui/SurfaceTool');

function StencilTextInsertTool() {
  StencilTextInsertTool.super.apply(this, arguments);
}

SurfaceTool.extend(StencilTextInsertTool);

StencilTextInsertTool.static.name = 'stencil-text-insert';
StencilTextInsertTool.static.command = 'stencil-text-insert';

module.exports = StencilTextInsertTool;
