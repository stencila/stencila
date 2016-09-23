'use strict';

import Emoji from './Emoji'
import EmojiHTMLConverter from './EmojiHTMLConverter'
import EmojiXMLConverter from './EmojiXMLConverter'
import EmojiComponent from './EmojiComponent'
import EmojiMarkdownComponent from './EmojiMarkdownComponent'
import EmojiCommand from './EmojiCommand'
import EmojiMacro from './EmojiMacro'
import EmojiTool from './EmojiTool'

module.exports = {
  name: 'emoji',
  configure: function (config) {
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
