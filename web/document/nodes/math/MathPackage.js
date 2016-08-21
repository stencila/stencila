'use strict';

var Math = require('./Math');
var MathHTMLConverter = require('./MathHTMLConverter');
var MathXMLConverter = require('./MathXMLConverter');
var MathComponent = require('./MathComponent');
var MathMarkdownComponent = require('./MathMarkdownComponent');
var MathCommand = require('./MathCommand');
var MathMacro = require('./MathMacro');
var MathTool = require('./MathTool');

module.exports = {
  name: 'math',
  configure: function(config) {
    config.addNode(Math);
    config.addConverter('html', MathHTMLConverter);
    config.addConverter('xml', MathXMLConverter);
    config.addComponent('math', MathComponent);
    config.addComponent('math-markdown', MathMarkdownComponent);
    config.addCommand('math', MathCommand);
    config.addMacro(new MathMacro());
    config.addTool('math', MathTool);
    // TODO
    // Choose/create a better math icon (this is a random temporary)
    config.addIcon('math', { 'fontawesome': 'fa-tree' });
    config.addLabel('math', {
      en: 'Math'
    });
  }
};
