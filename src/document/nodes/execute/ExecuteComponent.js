import Component from 'substance/ui/Component'
import CodeEditorComponent from '../../ui/CodeEditorComponent'

import math from '../../../utilities/math/index'

class ExecuteComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = super.render.call(this, $$)
      .addClass('sc-execute')
      .addClass(node.show ? 'sm-show' : '')
      .append(
        $$('div')
          .addClass('se-tools')
          .append(
            $$('button')
              .append(
                $$('i').addClass('fa fa-play')
              )
              .on('click', event => {
                node.refresh()
              }),
            $$('input')
              .attr({
                value: node.getCall(),
                placeholder: 'output = context(input, input, ...)',
                spellcheck: 'false'
              })
              .on('change', event => {
                node.setCall(event.target.value)
              })
          ),

        $$(ExecuteCodeEditorComponent, {
          node: node,
          codeProperty: 'code',
          languageProperty: 'context'
        }).ref('editor')
      )

    var errors = node.errors
    let session
    if (this.refs.editor) session = this.refs.editor.editor.getSession()
    if (errors && Object.keys(errors).length) {
      el.attr('data-status', 'error')

      let annotations = Object.keys(errors).map((row, index) => {
        return {
          row: row,
          column: 0,
          text: errors[row],
          type: 'error'
        }
      })
      if (session) session.setAnnotations(annotations)

      let $errors = $$('pre').addClass('se-errors')
      $errors.text(JSON.stringify(errors))
      el.append($errors)
    } else {
      if (session) session.clearAnnotations()
    }

    var results = node.results
    if (!this.output && results) {
      results.forEach(result => {
        let type = result.type
        let format = result.format
        let value = result.value
        let $result
        if (type === 'img') {
          if (format === 'svg') {
            $result = $$('div').html(value)
          } else {
            $result = $$('img').attr({
              src: value
            })
          }
        } else if (type === 'tab' && format === 'csv') {
          let table = ''
          value.split('\n').forEach(function (row) {
            table += '<tr>'
            row.split(',').forEach(function (cell) {
              table += '<td>' + cell + '</td>'
            })
            table += '</tr>'
          })
          $result = $$('table').html(table)
        } else if (type === 'dom' && format === 'html') {
          $result = $$('div').html(value)
        } else if (type === 'math') {
          $result = $$('div')
          try {
            $result.html(math.render(value, format))
          } catch (error) {
            $result.text(error.message)
          }
        } else {
          $result = $$('pre').text(value || '')
        }
        $result.addClass('se-result')
                .attr('data-type', type)
                .attr('data-format', format)
        el.append($result)
      })
    }

    return el
  }

  didMount () {
    let node = this.props.node

    node.on('changed', () => {
      this.rerender()
    })

    let editor = this.refs.editor.editor
    editor.commands.addCommand({
      name: 'reresh',
      bindKey: {win: 'Ctrl-Enter', mac: 'Command-Enter'},
      exec: editor => {
        node.refresh()
      },
      readOnly: true
    })
  }

}

/**
 * A code editor component which refreshes the node when the editor
 * looses focus. This is a temporary hack. Ideally, we want a refresh to occur
 * when the `node.code` changes, not when the editor looses focus.
 */
class ExecuteCodeEditorComponent extends CodeEditorComponent {
  _onEditorBlur () {
    super._onEditorBlur()
    this.props.node.refresh()
  }
}

export default ExecuteComponent
