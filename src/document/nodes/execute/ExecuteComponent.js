import Component from 'substance/ui/Component'
import CodeEditorComponent from '../../ui/CodeEditorComponent'

class ExecuteComponent extends Component {

  render ($$) {
    let node = this.props.node
    let el = super.render.call(this, $$)
      .addClass('sc-execute')
      .append(
        $$('div')
          .append(

            $$('input')
              .attr({
                value: node.language,
                placeholder: 'Execution language',
                spellcheck: 'false'
              })
              .on('change', (event) => {
                this.context.documentSession.transaction(function (tx, args) {
                  tx.set([node.id, 'language'], event.target.value)
                })
              }),

            $$('button')
              .text('refresh')
              .on('click', (event) => {
                node.refresh()
              }),

            $$('span')
              .text(node.duration.toString())
          ),

        $$(CodeEditorComponent, {
          node: node,
          codeProperty: 'source',
          languageProperty: 'language'
        })
      )

    var errors = node.result.errors
    if (errors) {
      el.append(
        $$('div').text(errors.toString())
      )
    }

    var output = node.result.output
    if (output) {
      let format = output.format
      let content = output.content
      let outputEl
      if (format === 'png') {
        outputEl = $$('img').attr({
          src: content
        })
      } else if (format === 'csv') {
        let table = ''
        content.split('\n').forEach(function (row) {
          table += '<tr>'
          row.split(',').forEach(function (cell) {
            table += '<td>' + cell + '</td>'
          })
          table += '</tr>'
        })
        outputEl = $$('table').html(table)
      } else {
        outputEl = $$('pre').text(content || '')
      }
      el.append(outputEl)
    }

    return el
  }

  didMount () {
    let node = this.props.node
    node.on('changed', () => {
      this.rerender()
    })
  }

}

export default ExecuteComponent
