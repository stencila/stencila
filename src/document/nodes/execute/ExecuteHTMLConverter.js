export default {

  type: 'execute',
  tagName: 'div',

  matchElement: function (el) {
    return el.attr('data-execute')
  },

  import: function (el, node, converter) {
    node.context = el.attr('data-execute')
    node.input = el.attr('data-input') || ''
    node.output = el.attr('data-output') || ''
    node.show = el.attr('data-show') || false
    node.extra = el.attr('data-extra')

    let $code = el.find('[data-code]')
    if ($code) {
      node.code = $code.text()
    }

    let $errors = el.find('[data-errors]')
    if ($errors) {
      node.errors = {
        '0': $errors.text()
      }
    }

    let $results = el.findAll('[data-result]')
    if ($results.length) {
      node.results = []
      $results.forEach(function ($result) {
        let result = {
          type: $result.attr('data-result'),
          format: $result.attr('data-format')
        }
        if (result.type === 'img') {
          if ($result.is('img')) {
            let src = $result.attr('src')
            let match = src.match('^data:image/([a-z]+);base64(.+)$')
            if (match) {
              result.format = match[1]
              result.value = src
            } else {
              result.format = 'url'
              result.value = src
            }
          } else {
            result.value = $result.html()
          }
        } else if (result.type === 'dom' && result.format === 'html') {
          result.value = $result.html()
        } else {
          result.value = $result.text()
        }
        node.results.push(result)
      })
    }
  },

  export: function (node, el, converter) {
    el.attr('data-execute', node.context)
    if (node.input) el.attr('data-input', node.input)
    if (node.output) el.attr('data-output', node.output)
    if (node.show) el.attr('data-show', node.show)
    if (node.extra) el.attr('data-extra', node.extra)

    let $$ = converter.$$
    if (node.code) {
      el.append(
        $$('pre')
          .attr('data-code', '')
          .text(node.code)
      )
    }
  }
}
