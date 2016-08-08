'use strict';

var Code = require('substance/packages/code/Code');
var CodeHTMLConverter = require('substance/packages/code/CodeHTMLConverter');
var CodeXMLConverter = require('substance/packages/code/CodeXMLConverter');
var AnnotationComponent = require('substance/ui/AnnotationComponent');
var AnnotationCommand = require('substance/ui/AnnotationCommand');
var AnnotationTool = require('substance/ui/AnnotationTool');
var CodeMacro = require('./CodeMacro');

module.exports = {
  name: 'code',
  configure: function(config) {
    config.addNode(Code);
    config.addConverter('html', CodeHTMLConverter);
    config.addConverter('xml', CodeXMLConverter);
    config.addComponent('code', AnnotationComponent);
    config.addCommand('code', AnnotationCommand, { nodeType: Code.type });
    config.addTool('code', AnnotationTool);
    config.addMacro(new CodeMacro());
    config.addIcon('code', { 'fontawesome': 'fa-code' });
    config.addStyle(__dirname, '_code.scss');
    config.addLabel('code', {
      en: 'Code',
      de: 'Code'
    });
  }
};
