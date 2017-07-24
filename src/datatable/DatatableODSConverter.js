import { DefaultDOMElement } from 'substance'

import DatatableConverter from './DatatableConverter'

/**
 * Converter to import/export a Datatable from/to an Open Document Spreadsheet (ODS) file
 *
 */
export default class DatatableODSConverter extends DatatableConverter {

  /**
   * @override
   */
  match (path) {
    let {ext} = this._parsePath(path)
    return Promise.resolve(ext === 'ods')
  }

  /**
   * @override
   */
  import (path, storer, buffer) { // eslint-disable-line
    throw new Error('DatatableODSConverter.import() not yet implemented')
  }

  /**
   * Helper method to convert an ODS `<office:document-content>` root element
   * into a Datatable `<datatable>` element.
   * 
   * @param  {string} content - XML string of the `content.xml` file
   * @return {DOMElement}
   */
  _importDatatableFromContent(content) {
    let {$datatable, $$} = this._importCreateElement()
    let $fields = $$('fields')
    let $values = $$('values')

    let $content = DefaultDOMElement.parseXML(content).getFirstChild()

    // Using a CSS selector here. e.g
    //    let $table = $content.find('office:body office:spreadsheet table:table')
    // does not work because namespaced tag names are not supported
    // Is there a better way to do this?
    let $body = $content.getChildren().filter(node => node.name === "office:body")[0]
    let $spreadsheet = $body.getChildren().filter(node => node.name === "office:spreadsheet")[0]
    let $table = $spreadsheet.getChildren().filter(node => node.name === "table:table")[0]

    let rowNum = 0
    for (let $tableRow of $table.getChildren().filter(node => node.name === "table:table-row")) {
      let headerRow = false
      let $row = $$('row')
      let colNum = 0
      for (let $tableCell of $tableRow.getChildren().filter(node => node.name === "table:table-cell")) {
        let value = $tableCell.text()
        let type = $tableCell.attr('office:value-type')

        // If the first colmn of the first row is a string then we
        // assume it is a header row and use it for field names
        if (!headerRow && rowNum === 0 && colNum === 0 && type === 'string') {
          headerRow = true
        }

        if (headerRow) {
          let $field = $$('field').attr('name', value)
          $fields.append($field)
        } else {
          let $value = $$('value').text(value)
          $row.append($value)
        }
        
        colNum = colNum + 1
      }
      if (!headerRow) $values.append($row)

      rowNum = rowNum + 1
    }
    $datatable.append($fields)
    $datatable.append($values)
   
    return $datatable
  }

  /**
   * @override
   */
  export (path, storer, buffer) { // eslint-disable-line
    throw new Error('DatatableODSConverter.export() not yet implemented')
  }

}
