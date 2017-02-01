import Component from 'substance/ui/Component'
import CodeEditorComponent from '../../ui/CodeEditorComponent'

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
          languageProperty: 'session'
        }).ref('editor')
      )

    var errors = node.result.errors
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
      session.setAnnotations(annotations)

      let errorsEl = $$('pre').addClass('se-errors')
      errorsEl.text(JSON.stringify(errors))
      el.append(errorsEl)
    } else {
      if (session) session.clearAnnotations()
    }

    var output = node.result.output
    if (output) {
      let type = output.type
      let format = output.format
      let value = output.value
      let outputEl
      if (format === 'png') {
        outputEl = $$('img').attr({
          src: value
        })
      } else if (format === 'csv') {
        let table = ''
        value.split('\n').forEach(function (row) {
          table += '<tr>'
          row.split(',').forEach(function (cell) {
            table += '<td>' + cell + '</td>'
          })
          table += '</tr>'
        })
        outputEl = $$('table').html(table)
      } else {
        outputEl = $$('pre').text(value || '')
      }
      outputEl.addClass('se-output')
              .attr('data-type', type)
              .append(
                $$('div').addClass('se-type')
                         .text(type)
              )
      el.append(outputEl)
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
