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
    }
    return el
  }

  _renderVegaLite() {
    renderVegaLite(this.props.value).then((output) => {
      this.setState({
        output
      })
    })
  }
}
