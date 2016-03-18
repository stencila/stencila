'use strict';

module.exports = {

  type: 'stencil-equation',
  tagName: 'div',

  matchElement: function(el) {
    return el.is('div[data-equation]');
  },

  import: function(el, node) {
    var source = el.find('script');
    if(source){
      var format = source.attr('type');
      format = format.split(';')[0];
      node.source = source.text();
      node.format = format;
    }
  },

  export: function(node, el, converter) {
    var $$ = converter.$$;
    el.attr('data-equation', 'true');
    el.append($$('script')
      .attr('type', node.format)
      .text(node.source)
    );
  }
};
