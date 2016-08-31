'use strict';

var uuid = require('substance/util/uuid');

var sanitize = require('./sanitize');

module.exports = {

  type: 'default',

  /**
   * Match all elements
   *
   * Caution! If this is not the last converter in the configurator
   * then this will probably match everything in the HTML document.
   */
  matchElement: function () {

    return true;

  },

  /**
   * Used by `DocumentHTMLImporter.defaultConverter` but given that
   * matching everything, it may not be necessary because we never get there!
   */
  createNodeData: function () {

    return {
      type: 'default',
      id: uuid('default')
    };

  },

  import: function (el, node) {

    node.html = el.outerHTML;

  },

  export: function (node, el, converter) {

    // Instead of using the prepopulated element,
    // we create a new one from the HTML stored in the node
    var wrapper = converter.$$('div');
    wrapper.html(sanitize(node.html));
    el = wrapper.children[0];
    el.attr('data-id', node.id);
    return el;

  }

};
