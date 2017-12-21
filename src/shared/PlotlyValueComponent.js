import { Component } from 'substance'
import { getFrameSize } from './cellHelpers'
import Plotly from 'plotly.js'

class PlotlyValueComponent extends Component {

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
      let size = getFrameSize(spec.layout)
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
        displayModeBar: false,
        showTips: true
      }
      spec.layout.width = size.width
      spec.layout.height = size.height
      let el = this.el.getNativeElement()
      Plotly.purge(el)
      Plotly.plot(el, spec.traces, spec.layout, options)
    }
  }
}

PlotlyValueComponent.isResizable = true

export default PlotlyValueComponent
