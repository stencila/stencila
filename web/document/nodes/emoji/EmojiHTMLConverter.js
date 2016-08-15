'use strict';

module.exports = {

  type: 'emoji',
  tagName: 'span',

  matchElement: function(el) {
    return el.is('[data-emoji]');
  },

  import: function(el, node, converter) {
    var match = el.text().match(/\:?([^\:]+)\:?/);
    node.name = match[1];
  },

  export: function(node, el, converter) {
    el.attr('data-emoji', '');
    el.text(':' + node.name + ':');
  }

};
