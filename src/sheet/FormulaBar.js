import { Component } from 'substance'
import CodeEditor from '../shared/CodeEditor'

export default class FormulaBar extends Component {

  render($$) {
    let node = this.props.node
    let el = $$('div').addClass('sc-formula-bar').append(
      $$('div').addClass('se-function-icon').append(
        $$('em').append(
          'ƒ',
          $$('sub').append('x')
        )
      ),
      $$(CodeEditor, {
        name: 'formula-bar',
        path: node.getPath(),
        excludedCommands: [],
        withoutBreak: true
      }).ref('cellEditor')
        .on('enter', this._onCodeEditorEnter)
        .on('escape', this._onCodeEditorEscape)
    )
    return el
  }

  getChildContext() {
    return this.props.context
  }

  _onCodeEditorEnter() {
    this.send('updateCell')
  }

  _onCodeEditorEscape() {
    this.send('cancelCellEditing')
  }

}
