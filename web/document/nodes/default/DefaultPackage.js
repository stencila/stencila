'use strict';

var Default = require('./Default');
var DefaultHTMLConverter = require('./DefaultHTMLConverter');
var DefaultXMLConverter = require('./DefaultXMLConverter');
var DefaultComponent = require('./DefaultComponent');

module.exports = {
  name: 'default',
  configure: function(config) {

    config.addNode(Default);
    config.addConverter('html', DefaultHTMLConverter);
    config.addConverter('xml', DefaultXMLConverter);
    config.addComponent('default', DefaultComponent);
    config.addIcon('default', { 'fontawesome': 'fa-circle-o' });
    config.addLabel('default', {
      en: 'Default node'
    });
  }
};
