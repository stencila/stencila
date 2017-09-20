import { NodeComponent } from 'substance'

export default class CellComponent extends NodeComponent {

  render($$) {
    let el = $$('div').addClass('sc-cell').append(
      'CELL_STUB'
    )
    return el
  }
}
