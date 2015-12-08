'use strict';

module.exports = {

  type: 'stencil-summary',
  tagName: 'div',

  matchElement: function(el) {
    return el.is('#summary, #description');
  },

  import: function(el, summary, converter) {
    summary.content = el.text();
  },

  export: function(summary, el, converter) {
    el.text(summary.content);
  }
};
