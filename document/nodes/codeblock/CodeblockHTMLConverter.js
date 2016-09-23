'use strict'

module.exports = {

  type: 'codeblock',
  tagName: 'pre',

  matchElement: function (el) {
    if (el.is('pre') && el.children.length === 1) {
      return el.children[0].is('code')
    }
    return false
  },

  import: function (el, node, converter) {
    var code = el.find('code')
    node.language = code.attr('class')
    node.source = code.text().trim()
  },

  export: function (node, el, converter) {
    el
    .append(
      converter.$$('code')
        .addClass(node.language)
        .text(node.source)
    )
  }
}
