import { Component } from 'substance'
import { render as renderVegaLite } from '../../util/viz/vegaLite'

export default class VegaLiteComponent extends Component {

  didMount() {
    this._renderVegaLite()
  }

  willReceiveProps() {
    this._renderVegaLite()
  }

  render($$) {
    let el = $$('div').addClass('sc-vega-lite')
    if (this.state.output) {
      el.html(this.state.output)
    } else if (this.state.error) {
      el.addClass('sm-error')
      if (this._lastOutput) {
        el.html(this._lastOutput)
      } else {
        el.text('ERROR')
      }
    }
    return el
  }

  _renderVegaLite() {
    try {
      renderVegaLite(this.props.value)
      .then((output) => {
        this.props.cell.clearRuntimeError('vega-lite')
        this._lastOutput = output
        this.setState({output})
      })
    } catch(error) {
      this.props.cell.addRuntimeError('vega-lite', error)
      this.setState({error})
    }
  }
}
