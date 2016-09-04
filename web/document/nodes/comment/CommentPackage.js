'use strict';

var Comment = require('./Comment');
var CommentHTMLConverter = require('./CommentHTMLConverter');
var CommentXMLConverter = require('./CommentXMLConverter');
var CommentComponent = require('./CommentComponent');
var CommentMarkdownComponent = require('./CommentMarkdownComponent');

module.exports = {
  name: 'comment',
  configure: function (config) {
    config.addNode(Comment);
    config.addConverter('html', CommentHTMLConverter);
    config.addConverter('xml', CommentXMLConverter);
    config.addComponent('comment', CommentComponent);
    config.addComponent('comment-markdown', CommentMarkdownComponent);
    config.addIcon('comment', { 'fontawesome': 'fa-comment' });
    config.addLabel('comment', {
      en: 'Comment'
    });
  }
};
