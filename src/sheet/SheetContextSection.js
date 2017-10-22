import { Component } from 'substance'

export default class SheetContextSection extends Component {

  render($$) {
    const contextId = this.props.contextId

    let el = $$('div').addClass('sc-context-section')

    if (!contextId) {
      console.error('FIXME: could not find contextId')
    } else {
      const ComponentClass = this.getComponent(contextId)

      el.append(
        $$(ComponentClass, { cellId: this.props.cellId })
      )
    }

    return el
  }
}
