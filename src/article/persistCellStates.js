import { forEach } from 'substance'

export default function persistCellStates (doc, dom) {
  let cells = doc.getIndex('type').get('cell')
  forEach(cells, cell => {
    let el = dom.find(`#${cell.id}`)
    let state = cell.state
    // store the cell output
    if (state.output) {
      el.attr('output-name', state.output)
    }
  })
}
