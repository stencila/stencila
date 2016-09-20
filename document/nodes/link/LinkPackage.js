'use strict';

import Link from 'substance/packages/link/Link'
var LinkComponent = require('./LinkComponent');
var LinkCommand = require('./LinkCommand');
var LinkTool = require('./LinkTool');
import LinkHTMLConverter from 'substance/packages/link/LinkHTMLConverter'
import LinkXMLConverter from 'substance/packages/link/LinkXMLConverter'
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
    config.addIcon('link', { 'fontawesome': 'fa-link' });
    config.addLabel('link', {
      en: 'Link',
      de: 'Link'
    });
  }
};
