'use strict';

var Mark = require('./Mark');
var MarkHTMLConverter = require('./MarkHTMLConverter');
var MarkXMLConverter = require('./MarkXMLConverter');
var MarkCommand = require('./MarkCommand');
var MarkComponent = require('./MarkComponent');
var AnnotationTool = require('substance/ui/AnnotationTool');

module.exports = {
  name: 'mark',
  configure: function (config) {

    config.addNode(Mark);
    config.addConverter('html', MarkHTMLConverter);
    config.addConverter('xml', MarkXMLConverter);
    config.addComponent('mark', MarkComponent);
    config.addCommand('mark', MarkCommand);
    config.addTool('mark', AnnotationTool);
    config.addIcon('mark', { 'fontawesome': 'fa-comment-o' });
    config.addLabel('mark', {
      en: 'Comment'
    });

  }
};
