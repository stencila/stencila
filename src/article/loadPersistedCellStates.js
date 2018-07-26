import { forEach, isNil } from 'substance'
import { OK, UNKNOWN } from '../engine/CellStates'
import CellState from '../engine/Cell'
import { getLang, getSource } from '../shared/cellHelpers'

export default function loadPersistedCellStates (doc) {
  let cells = doc.getIndex('type').get('cell')
  forEach(cells, cell => {
    let output = cell.find('output')
    let value
    if (output) {
      let json = output.textContent
      if (json) {
        value = JSON.parse(json)
      }
    }
    cell.state = new PseudoCellState(doc, cell, value)
  })
}

class PseudoCellState extends CellState {
  constructor (doc, cell, value) {
    super(doc, {
      id: cell.id,
      lang: getLang(cell),
      source: getSource(cell),
      value,
      status: !isNil(value) ? OK : UNKNOWN
    })
  }
}
