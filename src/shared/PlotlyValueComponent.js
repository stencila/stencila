import { Component } from 'substance'
import Plotly from 'plotly.js'

export default class PlotlyValueComponent extends Component {

  didMount() {
    this._renderPlotly()
  }

  willReceiveProps() {
    this._renderPlotly()
  }

  shouldRerender() {
    return false
  }

  render($$) {
    let el = $$('div').addClass('sc-plotly-value')
    return el
  }

  _renderPlotly() {
    if (this.el) {
      let value = this.props.value
      let spec = value.data
      let options = {
        // Find button names at
        // https://github.com/plotly/plotly.js/blob/master/src/components/modebar/buttons.js
        modeBarButtonsToRemove: [
          'sendDataToCloud',
          'autoScale2d',
          'hoverClosestCartesian', 'hoverCompareCartesian',
          'lasso2d', 'select2d'
        ],
        displaylogo: false,
        showTips: true
      }
      let el = this.el.getNativeElement()
      Plotly.purge(el)
      Plotly.plot(el, spec.traces, spec.layout, options)
    }
  }
}
