'use strict';

module.exports = {

  type: 'emoji',
  tagName: 'span',

  matchElement: function(el) {
    return el.is('span.emoji');
  },

  import: function(el, node, converter) {
    var match = el.text().match(/\:?([^\:]+)\:?/);
    node.name = match[1];
  },

  export: function(node, el, converter) {
    el.text(':' + node.name + ':');
  }

};
