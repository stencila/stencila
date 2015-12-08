'use strict';

module.exports = {

  type: 'stencil-title',
  tagName: 'div',

  matchElement: function(el) {
    return el.is('#title');
  },

  import: function(el, title, converter) {
    title.content = el.text();
  },

  export: function(title, el, converter) {
    el.text(title.content);
  }
};
