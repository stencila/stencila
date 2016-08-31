'use strict';

var Emphasis = require('substance/packages/emphasis/Emphasis');
var EmphasisHTMLConverter = require('substance/packages/emphasis/EmphasisHTMLConverter');
var EmphasisXMLConverter = require('substance/packages/emphasis/EmphasisXMLConverter');
var AnnotationComponent = require('substance/ui/AnnotationComponent');
var AnnotationCommand = require('substance/ui/AnnotationCommand');
var AnnotationTool = require('substance/ui/AnnotationTool');
var EmphasisMacro = require('./EmphasisMacro');

module.exports = {
  name: 'emphasis',
  configure: function (config) {

    config.addNode(Emphasis);
    config.addConverter('html', EmphasisHTMLConverter);
    config.addConverter('xml', EmphasisXMLConverter);
    config.addComponent('emphasis', AnnotationComponent);
    config.addCommand('emphasis', AnnotationCommand, { nodeType: 'emphasis' });
    config.addTool('emphasis', AnnotationTool);
    config.addMacro(new EmphasisMacro());
    config.addIcon('emphasis', { 'fontawesome': 'fa-italic' });
    config.addStyle(__dirname, '_emphasis.scss');
    config.addLabel('emphasis', {
      en: 'Emphasis',
      de: 'Betonung'
    });

  }
};
