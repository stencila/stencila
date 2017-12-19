import { ToggleTool/*, FontAwesomeIcon*/ } from 'substance'

export default class FunctionUsageTool extends ToggleTool {
  render($$) {
    let messages = this.props.commandState.messages
    let el = $$('div').addClass('sc-code-errors-tool')

    let errorsEl = $$('div').addClass('se-errors')
    if (messages.length > 0) {
      messages.forEach((err) => {
        errorsEl.append(
          $$('div').addClass('se-error').append(
            'Error: ',
            err.message
          )
        )
      })
    }
    el.append(errorsEl)
    return el
  }
}
