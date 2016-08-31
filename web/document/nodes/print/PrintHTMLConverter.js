'use strict';

/*
 * HTML converter for Print node.
 */
module.exports = {

  type: 'print',
  tagName: 'span',

  matchElement: function (el) {

    return el.is('[data-print]');

  },

  import: function (el, node, converter) {

    node.source = el.attr('data-print');
    node.error = el.attr('data-error');
    node.content = el.text();

  },

  export: function (node, el, converter) {

    el.attr('data-print', node.source);
    if (node.error) el.attr('data-error', 'true');
    el.text(node.output);

  }
};
