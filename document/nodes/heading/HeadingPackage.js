'use strict';

/**
 * A package for `Heading` nodes that is necessary (instead of using Substance's) to:
 *
 *  - add our own `HeadingComponent` class
 *  - provide a label for a plain old heading (ie. not numbered)
 */

import Heading from 'substance/packages/heading/Heading'
var HeadingComponent = require('./HeadingComponent');
var HeadingMarkdownComponent = require('./HeadingMarkdownComponent');
import HeadingHTMLConverter from 'substance/packages/heading/HeadingHTMLConverter'
import HeadingXMLConverter from 'substance/packages/heading/HeadingXMLConverter'
var HeadingMacro = require('./HeadingMacro');

module.exports = {
  name: 'heading',
  configure: function (config) {
    config.addNode(Heading);
    config.addComponent('heading', HeadingComponent);
    config.addComponent('heading-markdown', HeadingMarkdownComponent);
    config.addConverter('html', HeadingHTMLConverter);
    config.addConverter('xml', HeadingXMLConverter);
    config.addMacro(new HeadingMacro());
    config.addTextType({
      name: 'heading',
      data: {type: 'heading', level: 1}
    });
    config.addLabel('heading', {
      en: 'Heading',
      de: 'Überschrift'
    });
  }
};
