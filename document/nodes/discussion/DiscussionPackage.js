'use strict';

import Discussion from './Discussion'
import DiscussionHTMLConverter from './DiscussionHTMLConverter'
import DiscussionXMLConverter from './DiscussionXMLConverter'
import DiscussionComponent from './DiscussionComponent'
import DiscussionMarkdownComponent from './DiscussionMarkdownComponent'

module.exports = {
  name: 'discussion',
  configure: function (config) {
    config.addNode(Discussion);
    config.addConverter('html', DiscussionHTMLConverter);
    config.addConverter('xml', DiscussionXMLConverter);
    config.addComponent('discussion', DiscussionComponent);
    config.addComponent('discussion-markdown', DiscussionMarkdownComponent);
    config.addIcon('discussion', { 'fontawesome': 'fa-comments' });
    config.addLabel('discussion', {
      en: 'Discussion'
    });
  }
};
