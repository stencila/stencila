import { Component } from 'substance'
import CodeEditor from '../shared/CodeEditor'

export default class FormulaBar extends Component {

  render($$) {
    let node = this.props.node
    let el = $$('div').addClass('sc-edit-cell-expression-tool').append(
      $$('div').addClass('se-function-icon').append(
        $$('em').append(
          'ƒ',
          $$('sub').append('x')
        )
      ),
      $$(CodeEditor, {
        path: node.getPath(),
        excludedCommands: [],
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
    this.send('onCellEditorEnter')
  }

  _onCodeEditorEscape() {
    this.send('onCellEditorEscape')
  }

}
