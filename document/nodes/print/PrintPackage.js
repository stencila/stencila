'use strict';

var Print = require('./Print');
var PrintHTMLConverter = require('./PrintHTMLConverter');
var PrintXMLConverter = require('./PrintXMLConverter');
var PrintComponent = require('./PrintComponent');
var PrintCommand = require('./PrintCommand');
var PrintMacro = require('./PrintMacro');
var PrintTool = require('./PrintTool');

module.exports = {
  name: 'print',
  configure: function (config) {
    config.addNode(Print);
    config.addConverter('html', PrintHTMLConverter);
    config.addConverter('xml', PrintXMLConverter);
    config.addComponent('print', PrintComponent);
    config.addCommand('print', PrintCommand);
    config.addMacro(new PrintMacro());
    config.addTool('print', PrintTool);
    config.addIcon('print', { 'fontawesome': 'fa-eyedropper' });
    config.addLabel('print', {
      en: ''
    });
  }
};
