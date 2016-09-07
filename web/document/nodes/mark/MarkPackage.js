'use strict';

var Mark = require('./Mark');
var MarkHTMLConverter = require('./MarkHTMLConverter');
var MarkXMLConverter = require('./MarkXMLConverter');
var MarkCommand = require('./MarkCommand');
var MarkComponent = require('./MarkComponent');
var MarkTool = require('./MarkTool');

module.exports = {
  name: 'mark',
  configure: function (config) {
    config.addNode(Mark);
    config.addConverter('html', MarkHTMLConverter);
    config.addConverter('xml', MarkXMLConverter);
    config.addComponent('mark', MarkComponent);
    config.addCommand('mark', MarkCommand);
    config.addTool('mark', MarkTool);
    config.addIcon('mark', { 'fontawesome': 'fa-comment-o' });
    config.addLabel('mark', {
      en: 'Comment'
    });
  }
};
