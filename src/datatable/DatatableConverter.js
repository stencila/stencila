import { DefaultDOMElement } from 'substance'

import Converter from '../shared/Converter'

export default class DatatableConverter extends Converter {

  _importCreateElement () {
    let xml = DefaultDOMElement.createDocument('xml')
    let $$ = xml.createElement.bind(xml)
    let $datatable = $$('datatable')
    xml.append($datatable)

    return {
      $datatable,
      $$
    }
  }

  _importWriteBuffer ($datatable, buffer) {
    return buffer.writeFile('datatable.xml', $datatable.getOuterHTML() || '')
  }
}
