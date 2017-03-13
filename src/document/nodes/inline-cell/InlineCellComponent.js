import { Component, isNil } from 'substance'

class InlineCellComponent extends Component {

  didMount() {
    const node = this.props.node
    if (node) {
      node.on('value:updated', this.rerender, this)
    }
  }

  dispose() {
    const node = this.props.node
    if (node) {
      node.off(this)
    }
  }

  render($$) {
    let node = this.props.node
    let el = $$('span').addClass('sc-inline-cell')
    if (!isNil(node.value)) {
      // NOTE: caching the old value, so that we can
      // render it still while the engine is updating
      this._oldValue = node.value
      el.text(String(node.value))
    } else if (!isNil(this._oldValue)) {
      el.addClass('sm-pending')
      el.text(String(this._oldValue))
    }
    if (node.hasError()){
      el.text(String(node.getError()))
      el.addClass('sm-error')
    }
    return el
  }

}

export default InlineCellComponent
