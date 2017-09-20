import { Component } from 'substance'
import Plotly from 'plotly'

export default class PlotlyChart extends Component {
  render($$) {
    let el = $$('div').addClass('sc-plotly-chart')
    return el
  }

  didMount() {
    let el = this.el.getNativeElement()
    let data = [
      {
        x: [1, 2, 3, 4, 5],
        y: [1, 2, 4, 8, 16]
      }
    ]
    let layout = { margin: { t: 0 } }
    Plotly.plot(el, data, layout)
  }
}
