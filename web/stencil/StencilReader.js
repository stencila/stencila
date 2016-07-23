'use strict';

var StencilController = require('./StencilController');

var CONFIG = {
  controller: {
    commands: [
      require('substance-fe0ed/ui/UndoCommand'),
      require('substance-fe0ed/ui/RedoCommand'),
      require('substance-fe0ed/ui/SaveCommand'),
    ],
    components: {
      "paragraph": require('substance-fe0ed/packages/paragraph/ParagraphComponent'),
      "heading": require('substance-fe0ed/packages/heading/HeadingComponent'),
      "link": require('./packages/link/LinkComponent'),
      // Panels
      "toc": require('substance-fe0ed/ui/TOCPanel')
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
