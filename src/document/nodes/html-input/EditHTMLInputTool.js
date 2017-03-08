import { Tool, Component } from 'substance'

/*
  TODO: support other types than 'range' and 'number'
*/
class EditHTMLInputTool extends Tool {

  render($$) {
    let InputSettingsBar = this.getComponent('input-settings-bar')
    let el = $$('div').addClass('sc-edit-html-input-tool')

    el.append($$(InputSettingsBar, this.props))
    
    if (this.state.showSettings) {
      el.append($$(Settings, this.props))
    } else {
      el.append($$(Display, this.props))
    }
    return el
  }
}

class Settings extends Component {
  render($$) {
    let node = this.props.node
    let el = $$('div').addClass('se-settings')
    el.append('TODO: implement')
    return el
  }
}

class Display extends Component {
  render($$) {
    let node = this.props.node
    let el = $$('div').addClass('se-display')
    if (node.inputType === 'range' || node.inputType === 'number') {
      el.append(
        $$('input')
          .attr('type', node.inputType)
          .attr('min', node.min)
          .attr('max', node.max)
          .attr('step', node.step)
          .attr('value', node.value)
      )
    } else {
      el.append('Input type not yet supported')
    }
    return el
  }

  _onValueChanged(newValue) {
    this.context.editorSession.transaction((tx) => {
      tx.set([this.props.node.id, 'value'], newValue)
    })
    this.rerender()
  }
}

export default EditHTMLInputTool
