'use strict';

/**
 * A package for `Heading` nodes that is necessary (instead of using Substance's) to:
 *  
 *  - add our own `HeadingComponent` class
 *  - provide a label for a plain old heading (ie. not numbered)
 */

var Heading = require('substance/packages/heading/Heading');
var HeadingComponent = require('./HeadingComponent');
var HeadingHTMLConverter = require('substance/packages/heading/HeadingHTMLConverter');
var HeadingXMLConverter = require('substance/packages/heading/HeadingXMLConverter');
var HeadingMacro = require('./HeadingMacro');

module.exports = {
  name: 'heading',
  configure: function(config) {
    config.addNode(Heading);
    config.addComponent('heading', HeadingComponent);
    config.addConverter('html', HeadingHTMLConverter);
    config.addConverter('xml', HeadingXMLConverter);
    config.addMacro(new HeadingMacro());
    config.addTextType({
      name: 'heading',
      data: {type: 'heading', level: 1}
    });
    config.addStyle(__dirname, '_heading.scss');
    config.addLabel('heading', {
      en: 'Heading',
      de: 'Überschrift'
    });
  }
};
