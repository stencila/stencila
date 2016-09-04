'use strict';

var Title = require('./Title');
var TitleComponent = require('./TitleComponent');
var TitleHTMLConverter = require('./TitleHTMLConverter');
var TitleXMLConverter = require('./TitleXMLConverter');

module.exports = {
  name: 'title',
  configure: function (config) {
    config.addNode(Title);
    config.addComponent('title', TitleComponent);
    config.addConverter('html', TitleHTMLConverter);
    config.addConverter('xml', TitleXMLConverter);
    config.addTextType({
      name: 'title',
      data: {type: 'title'}
    });
    config.addIcon('title', { 'fontawesome': 'fa-asterisk' });
    config.addStyle(__dirname, '_title.scss');
    config.addLabel('title', {
      en: 'Title',
      de: 'Title'
    });
  }
};
