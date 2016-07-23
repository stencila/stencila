"use strict";

var SurfaceTool = require('substance-fe0ed/ui/SurfaceTool');

// TODO: this should go into substance/packages/table/
function InsertTableTool() {
  InsertTableTool.super.apply(this, arguments);
};

SurfaceTool.extend(InsertTableTool);

SurfaceTool.static.name = 'insertTable';

SurfaceTool.static.command = 'insertTable';

module.exports = InsertTableTool;
