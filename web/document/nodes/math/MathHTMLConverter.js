'use strict';

module.exports = {

  type: 'math',
  tagName: 'span',

  matchElement: function (el) {

    return el.is('[data-math]');

  },

  import: function (el, node, converter) {

    var spec = el.attr('data-math');
    var match = spec.match(/(\w+)(\s+(\w+))?/);
    node.language = match[1];
    node.display = match[3];
    node.source = el.text();

  },

  export: function (node, el, converter) {

    var spec = node.language;
    if (node.display === 'block') spec += ' block';
    el.attr('data-math', spec);
    el.text(node.source);

  }

};
