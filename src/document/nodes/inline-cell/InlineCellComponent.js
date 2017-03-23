import { Component, isNil } from 'substance'

class InlineCellComponent extends Component {

  didMount() {
    const node = this.props.node
    if (node) {
      node.on('evaluation:started', this.onEvaluationStarted, this)
      node.on('evaluation:finished', this.onEvaluationFinished, this)
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
    return el
  }

  onEvaluationStarted() {
    this.el.addClass('sm-pending')
  }

  onEvaluationFinished() {
    this.el.removeClass('sm-pending')
    this.rerender()
  }

}

export default InlineCellComponent
