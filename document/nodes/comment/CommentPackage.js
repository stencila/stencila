'use strict'

import Comment from './Comment'
import CommentHTMLConverter from './CommentHTMLConverter'
import CommentXMLConverter from './CommentXMLConverter'
import CommentComponent from './CommentComponent'
import CommentMarkdownComponent from './CommentMarkdownComponent'

module.exports = {
  name: 'comment',
  configure: function (config) {
    config.addNode(Comment)
    config.addConverter('html', CommentHTMLConverter)
    config.addConverter('xml', CommentXMLConverter)
    config.addComponent('comment', CommentComponent)
    config.addComponent('comment-markdown', CommentMarkdownComponent)
    config.addIcon('comment', { 'fontawesome': 'fa-comment' })
    config.addLabel('comment', {
      en: 'Comment'
    })
  }
}
