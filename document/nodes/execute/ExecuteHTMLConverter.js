'use strict'

module.exports = {

  type: 'execute',
  tagName: 'pre',

  matchElement: function (el) {
    return el.is('pre') && el.attr('data-execute')
  },

  import: function (el, node, converter) {
    var spec = el.attr('data-execute')
    var matches = spec.match(/(execute|r|py) *(show)?/)
    node.language = matches[1]
    node.show = matches[2]
    node.error = el.attr('data-error')
    node.extra = el.attr('data-extra')
    node.source = el.text()
  },

  export: function (node, el, converter) {
    var spec = node.language
    if (node.show) {
      spec += ' show'
    }
    el.attr('data-execute', spec)

    if (node.error) {
      el.attr('data-error', node.error)
    }

    if (node.extra) {
      el.attr('data-extra', node.extra)
    }

    el.text(node.source)
  }
}
