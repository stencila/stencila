export default {

  type: 'sheet-cell',
  tagName: 'td',

  matchElement: function(el) {
    return el.is('td')
  },

  import: function(el, node) {
    node.content = el.textContent
  },

  export: function(node, el) {
    el.textContent = node.content
  }
}
