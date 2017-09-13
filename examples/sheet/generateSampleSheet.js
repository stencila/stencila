import {DefaultDOMElement} from 'substance'

const template = `<?xml version="1.0"?>
<!DOCTYPE sheet PUBLIC "StencilaSheet 1.0" "StencilaSheet.dtd">
<sheet>
  <meta>
    <name>sample</name>
    <title>Untitled</title>
    <description>A sample Sheet</description>
    <columns></columns>
  </meta>
  <data></data>
</sheet>
`

export default function generateSampleSheet(rows, cols) {
  let doc = DefaultDOMElement.parseXML(template)
  // create column meta
  let columns = doc.find('columns')
  for (let j = 0; j < cols; j++) {
    let col = doc.createElement('col')
    col.setAttribute('name', `x${j+1}`)
    col.setAttribute('type', 'number')
    columns.append(col)
  }
  let data = doc.find('data')
  for (let i = 0; i < rows; i++) {
    let row = doc.createElement('row')
    for (let j = 0; j < cols; j++) {
      let cell = doc.createElement('cell')
      cell.append(String(Math.random()))
      row.append(cell)
    }
    data.append(row)
  }
  return doc
}