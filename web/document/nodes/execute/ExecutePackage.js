'use strict';

var Execute = require('./Execute');
var ExecuteComponent = require('./ExecuteComponent');
var ExecuteHTMLConverter = require('./ExecuteHTMLConverter');
var ExecuteXMLConverter = require('./ExecuteXMLConverter');

module.exports = {
  name: 'execute',
  configure: function (config) {

    config.addNode(Execute);
    config.addComponent('execute', ExecuteComponent);
    config.addConverter('html', ExecuteHTMLConverter);
    config.addConverter('xml', ExecuteXMLConverter);
    config.addTextType({
      name: 'execute',
      data: {type: 'execute'}
    });
    config.addIcon('execute', { 'fontawesome': 'fa-play' });
    config.addLabel('execute', {
      en: 'Execute'
    });

  }
};
