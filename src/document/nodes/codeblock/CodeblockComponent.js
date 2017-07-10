import { Component } from 'substance'
import CodeEditorComponent from '../../ui/CodeEditorComponent'

export default class CodeblockComponent extends Component {

  constructor (parent, props) {
    super(parent, props)
  }

  render ($$) {
    const node = this.props.node
    let el = $$('div').addClass('sc-codeblock')

    el.append($$(CodeEditorComponent, {
      path: [node.id, 'source'],
      languageProperty: 'language',
      disabled: this.props.disabled
    }))

    return el
  }

}

CodeblockComponent.fullWidth = true
