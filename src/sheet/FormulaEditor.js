import { Component } from 'substance'
import CodeEditor from '../shared/CodeEditor'

export default class FormulaEditor extends Component {

  render($$) {
    const node = this.props.context.node
    let el = $$('div').addClass('sc-formula-editor')
    el.append(
      $$(CodeEditor, {
        name: 'formula-editor',
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
