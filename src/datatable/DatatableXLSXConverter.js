import { DefaultDOMElement } from 'substance'

import DatatableConverter from './DatatableConverter'

/**
 * Converter to import/export a Datatable from/to an XSLX files
 *
 */
export default class DatatableXLSXConverter extends DatatableConverter {

  /**
   * @override
   */
  match (path) {
    let {ext} = this._parsePath(path)
    return Promise.resolve(ext === 'xlsx')
  }

  /**
   * @override
   */
  import (path, storer, buffer) { // eslint-disable-line
    throw new Error('DatatableXLSXConverter.import() not yet implemented')
  }

  /**
   * Helper method to convert an XSLX `<worksheet>` element
   * into a Datatable `<datatable>` element.
   * 
   * @param  {string} worksheet - XML string of the `xl/worksheets/sheet1.xml` file (or other sheet)
   * @param  {string} sharedStrings - XML string of the `xl/sharedStrings.xml` file
   * @return {DOMElement}
   */
  _importDatatableFromWorksheet(worksheet, sharedStrings) {
    let {$datatable, $$} = this._importCreateElement()
    let $fields = $$('fields')
    let $values = $$('values')

    let $worksheet = DefaultDOMElement.parseXML(worksheet).find('worksheet')
    
    // Create an array of strings to access as cell values
    let $sst = DefaultDOMElement.parseXML(sharedStrings).find('sst')
    let strings = $sst.getChildren().map($si => $si.find('t').text())

    let $sheetData = $worksheet.find('sheetData')
    let rowNum = 0
    for (let $row of $sheetData.findAll('row')) {
      let headerRow = false
      let $row_ = $$('row')
      let colNum = 0
      for (let $c of $row.findAll('c')) {
        let value = $c.text()
        let type = $c.attr('t')
        
        // If necessary convert the value to corresponding type
        if (type === 's') {
          value = strings[value]
        }

        // If the first colmn of the first row is a string then we
        // assume it is a header row and use it for field names
        if (!headerRow && rowNum === 0 && colNum === 0 && type === 's') {
          headerRow = true
        }

        if (headerRow) {
          let $field = $$('field').attr('name',value)
          $fields.append($field)
        } else {
          let $value = $$('value').text(value)
          $row_.append($value)
        }
        
        colNum = colNum + 1
      }
      if (!headerRow) $values.append($row_)

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
    throw new Error('DatatableXLSXConverter.export() not yet implemented')
  }

}
