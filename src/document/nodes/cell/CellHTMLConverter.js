import {type} from '../../../value.js'

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
    let $value = el.find('[data-value]')
    if ($value) {
      if ($value.tagName === 'img') {
        node.value = {
          type: 'image',
          src: $value.attr('src')
        }
      }
      else node.value = JSON.parse($value.textContent)
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
    let $value
    if (type(node.value) === 'image') {
      $value = $$('img').attr('src', node.value.src)
    } else {
      $value = $$('pre').text(JSON.stringify(node.value))
    }
    $value.attr('data-value', '')
    el.append($value)
  }

}
