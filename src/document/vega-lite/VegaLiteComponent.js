import { Component } from 'substance'
import { render as renderVegaLite } from '../../util/viz/vegaLite'

export default class VegaLiteComponent extends Component {

  didMount() {
    this._renderVegaLite()
  }

  willReceiveProps() {
    this._renderVegaLite()
  }

  shouldRerender() {
    return false
  }

  render($$) {
    let el = $$('div').addClass('sc-vega-lite')
    return el
  }

  _renderVegaLite() {
    try {
      renderVegaLite(this.props.value)
      .then(() => {
        // HACK somehow renderVegaLite always needs two runs to be correct
        renderVegaLite(this.props.value)
        .then((output) => {
          this.props.cell.clearRuntimeError('vega-lite')
          this.el.html(output)
        })
      })
    } catch(error) {
      this.props.cell.addRuntimeError('vega-lite', error)
    }
  }
}
