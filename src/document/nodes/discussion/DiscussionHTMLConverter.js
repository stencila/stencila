export default {

  type: 'discussion',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('[data-discussion]')
  },

  import: function (el, node, converter) {
    node.id = el.attr('id')
    el.getChildren().forEach(function (child) {
      node.nodes.push(converter.convertElement(child).id)
    })
  },

  export: function (node, el, converter) {
    el.attr('data-discussion', '')
    el.attr('id', node.id)
    node.getChildren().forEach(function (child) {
      el.append(converter.convertNode(child))
    })
  }

}
