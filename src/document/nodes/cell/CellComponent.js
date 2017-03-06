import { Component } from 'substance'
import TextInput from '../../../shared/substance/text-input/TextInput'

class CellComponent extends Component {

  render($$) {
    let node = this.props.node
    let el = $$('div').addClass('sc-cell')
    el.append(
      $$('div').addClass('se-expression').append(
        $$(TextInput, {
          content: node.expression
        }).ref('expressionEditor')
          .on('confirm', this.onConfirm)
          .on('cancel', this.onCancel)
      )
    )
    if (node.output) {
      el.append(
        $$('div').addClass('se-output').html(node.output)
      )
    }
    return el
  }

  getExpression() {
    return this.refs.expressionEditor.getContent()
  }

  onConfirm() {
    console.log('Yay')
    let newExpression = this.getExpression()
    this.context.editorSession.transaction((tx) => {
      tx.set([this.props.node.id, 'expression'], newExpression)
    })
  }

  onCancel() {
    console.log('cancelled')
  }

}

export default CellComponent
