import { Component } from 'substance'
import { getFrameSize } from './cellHelpers'
import Plotly from 'plotly.js'

export default class PlotlyValueComponent extends Component {

  didMount() {
    this._renderPlotly()
  }

  didUpdate() {
    this._renderPlotly()
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
        displayModeBar: false,
        showTips: true
      }
      // TODO: discuss. After some discussions, @integral and @oliver---
      // think that this component should not deal with sizes at all,
      // because it should come from the libcore function.
      // if the default values are not provided by the plot call
      // then we need to set default values here.
      // Note: in this call we make sure that there are default values set
      let size = getFrameSize(spec.layout)
      spec.layout.width = size.width
      spec.layout.height = size.height

      let el = this.el.getNativeElement()
      Plotly.purge(el)
      Plotly.plot(el, spec.traces, spec.layout, options)
    }
  }
}
