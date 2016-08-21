'use strict';

var Emoji = require('./Emoji');
var EmojiHTMLConverter = require('./EmojiHTMLConverter');
var EmojiXMLConverter = require('./EmojiXMLConverter');
var EmojiComponent = require('./EmojiComponent');
var EmojiMarkdownComponent = require('./EmojiMarkdownComponent');
var EmojiCommand = require('./EmojiCommand');
var EmojiMacro = require('./EmojiMacro');
var EmojiTool = require('./EmojiTool');

module.exports = {
  name: 'emoji',
  configure: function(config) {
    config.addNode(Emoji);
    config.addConverter('html', EmojiHTMLConverter);
    config.addConverter('xml', EmojiXMLConverter);
    config.addComponent('emoji', EmojiComponent);
    config.addComponent('emoji-markdown', EmojiMarkdownComponent);
    config.addCommand('emoji', EmojiCommand);
    config.addMacro(new EmojiMacro());
    config.addTool('emoji', EmojiTool);
    config.addIcon('emoji', { 'fontawesome': 'fa-smile-o' });
    config.addLabel('emoji', {
      en: 'Emoji'
    });
  }
};
