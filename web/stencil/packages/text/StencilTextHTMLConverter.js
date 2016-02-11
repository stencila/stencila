'use strict';

module.exports = {

  type: 'stencil-text',
  tagName: 'span',

  matchElement: function(el) {
    return el.is('[data-text]');
  },

  import: function(el, text, converter) {
    text.tagName = el.tagName;
    text.source = el.attr('data-text');
    text.error = el.attr('data-error');
    text.output = el.text();
  },

  export: function(text, el, converter) {
    el = el.withTagName(text.tagName);
    el.attr('data-text',text.source);
    el.text(text.output);
    if(text.error) el.attr('data-error',text.error);
    return el;
  }
};
