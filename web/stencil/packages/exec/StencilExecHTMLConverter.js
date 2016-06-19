'use strict';

module.exports = {

  type: 'stencil-exec',
  tagName: 'pre',

  matchElement: function(el) {
    return el.is('pre') && el.attr('data-exec');
  },

  import: function(el, node, converter) {
    var spec = el.attr('data-exec');
    var matches = spec.match(/(exec|r|py) *(show)?/);
    node.lang = matches[1];
    node.show = matches[2]?true:false;
    node.spec = spec;
    node.error = el.attr('data-error');
    node.extra = el.attr('data-extra');
    node.source = el.text();
  },

  export: function(node, el, converter) {
    el
     .attr('data-exec', node.spec)
     .text(node.source);
    if (node.error) {
      el.attr('data-error', node.error);
    }
    if (node.extra) {
      el.attr('data-extra', node.extra);
    }
  }
};
