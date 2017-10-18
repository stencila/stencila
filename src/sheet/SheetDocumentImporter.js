import { XMLDocumentImporter } from 'substance'

export default
class SheetDocumentImporter extends XMLDocumentImporter {

  /*
    overridden to enforce some ids for singular elements, such as
    the root element, or its data element
  */
  _getIdForElement(el, type) {
    switch (type) {
      case 'sheet':
      case 'data':
      case 'columns':
        return type
    }
    return super._getIdForElement(el, type)
  }
}