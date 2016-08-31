'use strict';

var sanitizeHtml = require('sanitize-html');

/**
 * Apply HTML sanitization rules for `Default` nodes
 *
 * @param      {<type>}  html    The html
 */
var sanitize = function (html) {

  return sanitizeHtml(html, {
    allowedTags: [
      'b', 'i', 'strong', 'em', 'strike', 'code', 'hr', 'br',
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'div', 'p', 'pre', 'blockquote',
      'span', 'a', 'img',
      'ul', 'ol', 'li',
      'table', 'thead', 'caption', 'tbody', 'tr', 'th', 'td',
      'figure', 'figcaption'
    ],
    allowedAttributes: {
      a: [ 'href', 'name', 'target', 'data-*' ],
      img: [ 'src', 'data-*' ],
      '*': [ 'data-*' ]
    }
  });

};

module.exports = sanitize;

