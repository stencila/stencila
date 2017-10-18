import { NodeComponent } from 'substance'

export default class ReproFigComponent extends NodeComponent {

  render($$) {
    let el = $$('div').addClass('sc-repro-fig').append(
      'REPRO_FIG_STUB'
    )
    return el
  }
}
