'use strict';

module.exports = {

  type: 'stencil-exec',
  tagName: 'script',

  matchElement: function(el) {
    return el.is('script[type^="math/tex"],script[type^="math/asciimath"]');
  },

  import: function(el, math, converter) {
    var format = el.attr('type');
    math.format = format.split(';')[0];
    math.source = el.text();
  },

  export: function(math, el, converter) {
    el
      .prop('type', math.format)
      .text(math.source);
  }
};
