import {DefaultDOMElement} from 'substance'

export default function template(colSpecs = {}, cells = {}, cols = 20, rows = 100) {
  let doc = DefaultDOMElement.parseXML(`<?xml version="1.0"?>
  <!DOCTYPE sheet PUBLIC "StencilaSheet 1.0" "StencilaSheet.dtd">
  <sheet>
    <meta>
      <name>template</name>
      <title>Blank sheet</title>
      <description>An example sheet that is blank</description>
      <columns></columns>
    </meta>
    <data></data>
  </sheet>
  `)

  let columnsEl = doc.find('columns')
  for (let col = 0; col < cols; col++) {
    let colEl = doc.createElement('col')
    let colSpec = colSpecs[col]
    if (colSpec) {
      colEl.setAttribute('name', colSpec.name)
      colEl.setAttribute('type', colSpec.number)
    }
    columnsEl.append(colEl)
  }

  let data = doc.find('data')
  for (let col = 0; col < rows; col++) {
    let row = doc.createElement('row')
    for (let col = 0; col < cols; col++) {
      let cell = doc.createElement('cell')
      let content = ''
      cell.append(content)
      row.append(cell)
    }
    data.append(row)
  }

  return doc
}
