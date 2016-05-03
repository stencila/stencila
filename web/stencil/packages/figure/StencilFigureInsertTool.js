"use strict";

var SurfaceTool = require('substance/ui/SurfaceTool');

function StencilFigureInsertTool() {
  StencilFigureInsertTool.super.apply(this, arguments);
}

SurfaceTool.extend(StencilFigureInsertTool);

StencilFigureInsertTool.static.name = 'stencil-figure-insert';
StencilFigureInsertTool.static.command = 'stencil-figure-insert';

module.exports = StencilFigureInsertTool;
