import { Component } from 'substance'
import { render as renderVegaLite } from '../../util/viz/vegaLite'

export default class VegaLiteComponent extends Component {
  render($$) {
    let el = $$('div').addClass('sc-vega-lite')
    return el
  }
}
