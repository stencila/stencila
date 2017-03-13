import { Tool, Component } from 'substance'

/*
  TODO: support other types than 'range' and 'number'
*/
class EditRangeInputTool extends Tool {
  constructor(...args) {
    super(...args)
  }
  render($$) {
    let InputSettingsBar = this.getComponent('input-settings-bar')
    let el = $$('div').addClass('sc-edit-range-input-tool')
    el.append(
      $$(InputSettingsBar, this.props)
        .on('toggle', this._onToggleSettings)
    )
    if (this.state.showSettings) {
      el.append($$(Settings, this.props))
    } else {
      el.append($$(Display, this.props))
    }
    return el
  }

  _onToggleSettings() {
    this.setState({
      showSettings: !this.state.showSettings
    })
  }
}

class Settings extends Component {
  render($$) {
    let el = $$('div').addClass('se-settings')
    el.append(
      this.renderParamEditor($$, 'min', 'Minimum'),
      this.renderParamEditor($$, 'max', 'Maximum'),
      this.renderParamEditor($$, 'step', 'Step'),
      this.renderParamEditor($$, 'value', 'Default')
    )
    return el
  }

  renderParamEditor($$, name, label) {
    let el = $$('div').addClass('se-param-'+name)
    el.append(
      $$('label').append(label),
      $$('input')
        .addClass('st-tiny-input')
        .attr('type', 'number')
        .val(this.props.node[name])
        .ref(name)
        .on('change', this._onParamChanged.bind(this, name))
    )
    return el
  }

  _onParamChanged(name) {
    let newValue = this.refs[name].val()
    this.context.editorSession.transaction((tx) => {
      tx.set([this.props.node.id, name], Number(newValue))
    }, {
      // TODO: it would be desirable to activate this, since
      // currently the cursor disappears while editing and saving
      // parameters
      // skipSelectionRerender: true
    })
  }
}

class Display extends Component {
  render($$) {
    let node = this.props.node
    let el = $$('div').addClass('se-display')
    if (node.inputType === 'range' || node.inputType === 'number') {
      el.append(
        $$('input')
          .ref('value')
          .attr('type', node.inputType)
          .attr('min', node.min)
          .attr('max', node.max)
          .attr('step', node.step)
          .attr('value', node.value)
          .on('input', this._onValueInput)
          .on('change', this._onValueChanged),
        $$('span')
          .addClass('se-value-preview')
          .ref('valuePreview')
          .append(node.value)
      )
    } else {
      el.append('Input type not yet supported')
    }
    return el
  }

  _onValueInput() {
    let newValue = this.refs.value.val()
    this.refs.valuePreview.getNativeElement().textContent = newValue
  }

  _onValueChanged() {
    let newValue = this.refs.value.val()
    this.context.editorSession.transaction((tx) => {
      tx.set([this.props.node.id, 'value'], Number(newValue))
    }, {
      // skipSelectionRerender: true
    })
  }
}

export default EditRangeInputTool
