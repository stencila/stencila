'use strict';

module.exports = {

  type: 'discussion',
  tagName: 'div',

  matchElement: function (el) {

    return el.is('[data-discussion]');

  },

  import: function (el, node, converter) {

    var id = el.attr('id');
    node.id = id;
    // TODO
    // Only import comments
    el.getChildren().forEach(function (child) {

      node.nodes.push(converter.convertElement(child).id);

    });

  },

  export: function (node, el, converter) {

    el.text(node.content);

  }

};
