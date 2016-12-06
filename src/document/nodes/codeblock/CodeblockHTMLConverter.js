export default {

  type: 'codeblock',
  tagName: 'pre',

  matchElement: function (el) {
    if (el.is('pre') && el.children.length === 1) {
      return el.children[0].is('code')
    }
    return false
  },

  import: function (el, node, converter) {
    node.language = el.attr('class') || 'text'
    var code = el.find('code')
    node.source = code.text().trim()
  },

  export: function (node, el, converter) {
    if (node.language && node.language !== 'text') el.addClass(node.language)
    el.append(
      converter.$$('code')
        .text(node.source)
    )
  }
}
