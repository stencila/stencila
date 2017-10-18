import { Component } from 'substance'
import Plotly from 'plotly'

export default class PlotlyValueComponent extends Component {
  render($$) {
    let el = $$('div').addClass('sc-plotly-value')
    return el
  }

  didMount() {
    let value = this.props.value
    let spec = value.data
    let options = { 
      // Find button names at https://github.com/plotly/plotly.js/blob/master/src/components/modebar/buttons.js
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
    Plotly.plot(el, spec.traces, spec.layout, options)
  }
}
