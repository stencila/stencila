export default {

  type: 'cell',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('div[data-cell]')
  },

  import: function (el, node) {
    node.expression = el.attr('data-cell')
    let sourceCodeEl = el.find('pre[data-source]')
    if (sourceCodeEl) {
      node.sourceCode = sourceCodeEl.textContent
    }
    let outputEl = el.find('pre[data-output]')
    if (outputEl) {
      node.value = JSON.parse(outputEl.textContent)
    }
  },

  export: function (node, el, converter) {
    let $$ = converter.$$
    el.attr('data-cell', node.expression)
    if (node.sourceCode) {
      el.append(
        $$('pre').attr('data-source', '').text(node.sourceCode)
      )
    }
    el.append(
      $$('pre').attr('data-output', '').text(JSON.stringify(node.value))
    )
  }

}
