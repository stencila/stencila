import { DefaultDOMElement as DOM, isString } from 'substance'

const EMPTY_SHEET = `<?xml version="1.0"?>
<!DOCTYPE sheet PUBLIC "StencilaSheet 1.0" "StencilaSheet.dtd">
<sheet>
  <meta>
    <name></name>
    <title></title>
    <description></description>
    <columns>
    </columns>
  </meta>
  <data>
  </data>
</sheet>`

/*
  A simplified factory method to create a Sheet XML.
*/
export default function createSheetXML(spec) {
  let doc = DOM.parseXML(EMPTY_SHEET)
  const $$ = doc.createElement.bind(doc)
  let ncols
  if (spec.columns) {
    let columns = doc.find('columns')
    spec.columns.forEach(colSpec => {
      const { name, type } = colSpec
      let col = $$('col')
      if (name) col.attr('name', name)
      if (type) col.attr('type', type)
      columns.append(col)
    })
    ncols = spec.columns.length
  }
  if (spec.cells) {
    let data = doc.find('data')
    spec.cells.forEach(rowSpec => {
      if (!ncols) ncols = rowSpec.length
      if (ncols !== rowSpec.length) throw new Error('Illegal number of cells.')
      let row = $$('row')
      rowSpec.forEach(cellSpec => {
        let cell = $$('cell')
        let source, id, type
        if (isString(cellSpec)) {
          source = cellSpec
        } else {
          ({ id, type, source } = cellSpec)
        }
        if (id) cell.attr('id', id)
        if (type) cell.attr('type', type)
        cell.append(source)
        row.append(cell)
      })
      data.append(row)
    })
  }
  if (!spec.columns) {
    let columns = doc.find('columns')
    for (let i = 0; i < ncols; i++) {
      columns.append($$('col').attr('type', 'any'))
    }
  }
  return doc.serialize()
}