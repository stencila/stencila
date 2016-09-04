'use strict';

var Link = require('substance/packages/link/Link');
var LinkComponent = require('./LinkComponent');
var LinkCommand = require('./LinkCommand');
var LinkTool = require('./LinkTool');
var LinkHTMLConverter = require('substance/packages/link/LinkHTMLConverter');
var LinkXMLConverter = require('substance/packages/link/LinkXMLConverter');
var LinkMacro = require('./LinkMacro');

module.exports = {
  name: 'link',
  configure: function (config) {
    config.addNode(Link);
    config.addComponent('link', LinkComponent);
    config.addConverter('html', LinkHTMLConverter);
    config.addConverter('xml', LinkXMLConverter);
    config.addCommand('link', LinkCommand, {nodeType: 'link'});
    config.addTool('link', LinkTool);
    config.addMacro(new LinkMacro());
    config.addStyle(__dirname, '_link.scss');
    config.addIcon('link', { 'fontawesome': 'fa-link' });
    config.addLabel('link', {
      en: 'Link',
      de: 'Link'
    });
  }
};
