'use strict';

var Image = require('substance/packages/image/Image');
var ImageComponent = require('substance/packages/image/ImageComponent');
var ImageMarkdownComponent = require('./ImageMarkdownComponent');
var ImageHTMLConverter = require('substance/packages/image/ImageHTMLConverter');
var ImageXMLConverter = require('substance/packages/image/ImageXMLConverter');
var ImageMacro = require('./ImageMacro');
var ImageTool = require('./ImageTool');

module.exports = {
  name: 'image',
  configure: function (config) {
    config.addNode(Image);
    config.addComponent('image', ImageComponent);
    config.addComponent('image-markdown', ImageMarkdownComponent);
    config.addConverter('html', ImageHTMLConverter);
    config.addConverter('xml', ImageXMLConverter);
    config.addMacro(new ImageMacro());
    config.addTool('image', ImageTool);
    config.addIcon('image', { 'fontawesome': 'fa-image' });
    config.addLabel('image', {
      en: 'Image',
      de: 'Überschrift'
    });
  }
};
