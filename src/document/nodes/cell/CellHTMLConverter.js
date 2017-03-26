export default {

  type: 'cell',
  tagName: 'div',

  matchElement: function (el) {
    return el.is('div[data-cell]')
  },

  import: function (el, node) {
    const language = el.attr('data-language')
    if (language) {
      node.language = language
    }
    node.expression = el.attr('data-cell')
    let sourceCodeEl = el.find('pre[data-source]')
    if (sourceCodeEl) {
      node.sourceCode = sourceCodeEl.textContent
    }
    // TODO: discuss how we want to do this now:
    // let outputEl = el.find('pre[data-output]')
    // if (outputEl) {
    //   node.value = JSON.parseoutputEl.textContent
    // }
  },

  // TODO: This code has not yet been tested
  export: function (node, el, converter) {
    let $$ = converter.$$
    el.attr('data-cell', node.expression)
    if (node.sourceCode) {
      el.attr('data-language', node.language)
      el.append(
        $$('pre').attr('data-source', '').text(node.sourceCode)
      )
    }
    // TODO: discuss how we want to do this now:
    // to render in the same way as we do it in CellValueComponent
    // el.append(
    //   $$('pre').attr('data-output', '').text(node.value)
    // )
  }

}
