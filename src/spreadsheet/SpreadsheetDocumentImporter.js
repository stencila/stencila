import { XMLDocumentImporter } from 'substance'

export default
class SpreadsheetDocumentImporter extends XMLDocumentImporter {

  /*
    overridden to enforce some ids for singular elements, such as
    the root element, or its data element
  */
  _getIdForElement(el, type) {
    switch (type) {
      case 'spreadsheet':
      case 'data':
      case 'columns':
        return type
    }
    return super._getIdForElement(el, type)
  }
}