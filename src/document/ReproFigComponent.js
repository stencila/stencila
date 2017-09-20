import { NodeComponent } from 'substance'
import PlotlyChart from '../shared/PlotlyChart'

export default class ReproFigComponent extends NodeComponent {

  render($$) {
    let el = $$('div').addClass('sc-repro-fig').append(
      'REPRO_FIG_STUB',
      $$(PlotlyChart)
    )
    return el
  }
}
