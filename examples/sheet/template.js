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
    let colId = String.fromCharCode(65 + col)
    let colSpec = colSpecs[colId]
    if (colSpec) {
      colEl.setAttribute('name', colSpec.name)
      colEl.setAttribute('type', colSpec.type)
    }
    columnsEl.append(colEl)
  }

  let data = doc.find('data')
  for (let row = 1; row <= rows; row++) {
    let rowEl = doc.createElement('row')
    for (let col = 0; col < cols; col++) {
      let cellEl = doc.createElement('cell')
      let cellId = String.fromCharCode(65 + col) + row
      let content = cells[cellId] || ''
      cellEl.append(content)
      rowEl.append(cellEl)
    }
    data.append(rowEl)
  }

  return doc
}
