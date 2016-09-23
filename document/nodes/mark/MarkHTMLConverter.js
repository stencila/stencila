'use strict'

export default {

  type: 'mark',
  tagName: 'span',

  matchElement: function (el) {
    return el.is('[data-mark]')
  },

  import: function (el, node, converter) {
    node.target = el.attr('data-mark')
  },

  export: function (node, el, converter) {
    el.attr('data-mark', node.target)
  }

}
