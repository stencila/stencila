'use strict';


module.exports = {

  type: 'math',
  tagName: 'script',

  matchElement: function(el) {
    return el.is('script[type^="math/"]');
  },

  import: function(el, node, converter) {
    var type = el.attr('type');
    var match = type.match(/math\/(\w+)(\;display=(\w+))?/);
    node.language = match[1];
    node.display = match[3];
    node.source = el.text();
  },

  export: function(node, el, converter) {
    var type = 'math/' + node.language;
    if (node.display === 'block') type += ';display=block';
    el.attr('type', type);
    el.text(node.source);
  }

};
