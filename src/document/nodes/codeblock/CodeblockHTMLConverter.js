export default {

  type: 'codeblock',
  tagName: 'pre',

  matchElement: function (el) {
    if (el.is('pre') && el.children.length === 1) {
      return el.children[0].is('code')
    }
    return false
  },

  import: function (el, node) {
    let codeEl = el.find('code')
    if (codeEl) {
      node.language = codeEl.attr('data-language') || 'text'
      node.source = codeEl.text().trim()
    }
  },

  export: function (node, el) {
    let codeEl = el.createElement('code').text(node.source)
    if (node.language) {
      codeEl.attr('data-language', node.language)
    }
    el.append(codeEl)
  }
}