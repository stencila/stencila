import Component from 'substance/ui/Component'

import code from '../../../utilities/code/index'

class CodeSubcomponent extends Component {

  constructor (...args) {
    super(...args)

    this.editor = null
  }

  render ($$) {
    return $$('pre')
      .addClass('sc-execution-code')
      .ref('pre')
  }

  shouldRerender () {
    return false
  }

  didMount () {
    code.loadAce()
    code.attachAceEditor(this.refs.pre.getNativeElement(), '', {
      language: 'sql', // TODO: set based on the current session language
      minLines: 1
    }, editor => {
      editor.commands.addCommand({
        name: 'execute',
        bindKey: {win: 'Ctrl-Enter', mac: 'Command-Enter'},
        exec: editor => this.send('execute')
      })

      editor.commands.addCommand({
        name: 'back',
        bindKey: {win: 'Ctrl-Up', mac: 'Command-Up'},
        exec: editor => this.send('back')
      })

      editor.commands.addCommand({
        name: 'forward',
        bindKey: {win: 'Ctrl-Down', mac: 'Command-Down'},
        exec: editor => this.send('forward')
      })

      this.editor = editor
    })
  }

}

class ExecutionComponent extends Component {

  render ($$) {
    let node = this.props.node

    let el = $$('div')
      .addClass('sc-execution')

    let input = $$('div').append(
      $$('select')
        .ref('inputType')
        .append(
          $$('option').attr('value', 'bool').text('Boolean'),
          $$('option').attr('value', 'int').text('Integer'),
          $$('option').attr('value', 'table').text('Table')
        ),
      $$('select')
        .ref('inputFormat')
        .append(
          $$('option').attr('value', 'str').text('String'),
          $$('option').attr('value', 'csv').text('CSV')
        ),
      $$('textarea')
        .ref('inputValue')
        .addClass('se-execution-input')
    )

    let code = $$(CodeSubcomponent, node.code).ref('code')

    let output = $$('div')
      .ref('output')
      .addClass('se-execution-output')

    let result = node.result
    if (result) {
      let errors = result.errors
      if (errors) {
        // If errors in the result then display as annotations in code...
        let session = this.refs.code.editor.getSession()
        let annotations = Object.keys(errors).map((row, index) => {
          return {
            row: row,
            column: 0,
            text: errors[row],
            type: 'error'
          }
        })
        session.setAnnotations(annotations)
        ///... and in output
        output.append(
          $$('pre').text(JSON.stringify(errors))
        )
      } else {
        // Otherwise, display the content
        let out = result.output
        if (out) {
          let type = out.type
          let format = out.format
          let value = out.value
          output.append(
            $$('div').text(type || ''),
            $$('div').text(format || '')
          )
          if (format === 'png') {
            output.append(
              $$('img').attr('src', value)
            )
          } else {
            output.append(
              $$('pre').text(value || '')
            )
          }
        }
      }
    }

    el.append(input, code, output)
    return el
  }

}

export default ExecutionComponent

