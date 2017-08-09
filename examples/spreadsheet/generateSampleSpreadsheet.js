import {DefaultDOMElement} from 'substance'

const template = `<?xml version="1.0"?>
<!DOCTYPE spreadsheet PUBLIC "StencilaSpreadsheet 1.0" "StencilaSpreadsheet.dtd">
<spreadsheet>
  <meta>
    <name>sample</name>
    <title>Untitled</title>
    <description>A sample Spreadsheet</description>
    <columns></columns>
  </meta>
  <data></data>
</spreadsheet>
`

export default function generateSampleSheet(rows, cols) {
  let doc = DefaultDOMElement.parseXML(template)
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