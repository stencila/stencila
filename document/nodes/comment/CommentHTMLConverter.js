'use strict'

export default {

  type: 'comment',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('[data-comment]')
  },

  import: function (el, node, converter) {
    var spec = el.attr('data-comment')
    var matches = spec.match(/([\w@]+)?(\s+at\s+(\S+))?/)
    node.who = matches[1]
    node.when = matches[3]
    el.getChildren().forEach(function (child) {
      node.nodes.push(converter.convertElement(child).id)
    })
  },

  export: function (node, el, converter) {
    var spec = node.who + ' at ' + node.when
    el.attr('data-comment', spec)
    node.getChildren().forEach(function (child) {
      el.append(converter.convertNode(child))
    })
  }

}
