'use strict';

var StencilController = require('./StencilController');

var CONFIG = {
  controller: {
    commands: [
      require('substance/ui/UndoCommand'),
      require('substance/ui/RedoCommand'),
      require('substance/ui/SaveCommand'),
    ],
    components: {
      "paragraph": require('substance/packages/paragraph/ParagraphComponent'),
      "heading": require('substance/packages/heading/HeadingComponent'),
      "link": require('./packages/link/LinkComponent'),
      // Panels
      "toc": require('substance/ui/TOCPanel')
    }
  },
  body: {
    commands: [],
  },
  panels: {
    'toc': {
      hideContextToggles: true
    }
  },
  tabOrder: ['toc'],
  containerId: 'body',
  isEditable: false
};

function StencilReader() {
  StencilReader.super.apply(this, arguments);
}

StencilReader.Prototype = function() {};

StencilController.extend(StencilReader);

StencilReader.static.config = CONFIG;

module.exports = StencilReader;
