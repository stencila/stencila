'use strict';

var ControllerTool = require('substance/ui/ControllerTool');
var RenderTool = ControllerTool.extend({

  static: {
    name: 'render',
    command: 'render'
  }

});

module.exports = RenderTool;
