export default {

  type: 'input',
  tagName: 'input',

  matchElement: function (el) {
    return el.is('input')
  },

  import: function (el, node, converter) {
    node.name = el.attr('name')
    node.displayType = el.attr('type') || 'text'
    node.dataType = el.attr('data-type') || 'text'
  },

  export: function (node, el, converter) {
    el.attr({
      name: node.name,
      type: node.displayType,
      'data-type': node.dataType
    })
  }
}
