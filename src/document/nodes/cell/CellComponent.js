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
        }).ref('editor')
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

  onConfirm() {
    console.log('Yay')
  }

  onCancel() {
    console.log('cancelled')
  }

}

export default CellComponent
