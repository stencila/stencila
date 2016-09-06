'use strict';

var Strong = require('substance/packages/strong/Strong');
var StrongHTMLConverter = require('substance/packages/strong/StrongHTMLConverter');
var StrongXMLConverter = require('substance/packages/strong/StrongXMLConverter');
var AnnotationComponent = require('substance/ui/AnnotationComponent');
var AnnotationCommand = require('substance/ui/AnnotationCommand');
var AnnotationTool = require('substance/ui/AnnotationTool');
var StrongMacro = require('./StrongMacro');

module.exports = {
  name: 'strong',
  configure: function (config) {
    config.addNode(Strong);
    config.addConverter('html', StrongHTMLConverter);
    config.addConverter('xml', StrongXMLConverter);
    config.addComponent('strong', AnnotationComponent);
    config.addCommand('strong', AnnotationCommand, { nodeType: 'strong' });
    config.addTool('strong', AnnotationTool);
    config.addMacro(new StrongMacro());
    config.addIcon('strong', { 'fontawesome': 'fa-bold' });
    config.addLabel('strong', {
      en: 'Strong',
      de: 'Starke'
    });
  }
};
