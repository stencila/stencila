import { XMLDocumentImporter, DefaultDOMElement } from 'substance'

export default class FunctionDocumentImporter extends XMLDocumentImporter {

  /**
   * Compile an XML document from the main XML files and any other
   * sibling files that are "include"ed.
   *
   * @param  {String} xml A string of the XML of the main function definition file
   * @param  {Object} files A dictionary of sibling file names and content
   */
  compileDocument(xml, files) {
    let doc = DefaultDOMElement.parseXML(xml)
    if (files) {
      for (let $code of doc.findAll('code[include]')) {
        let file = $code.attr('include')
        let code = files[file]
        if (!code) throw new Error(`File "${file}" to be included as in Function definition was not supplied`)
        $code.text(code)
      }
    }
    return doc
  }

  /*
    overridden to enforce some ids for singular elements
  */
  _getIdForElement(el, type) {
    switch (type) {
      case 'function':
      case 'name':
      case 'title':
      case 'summary':
      case 'params':
      case 'return':
      case 'implems':
      case 'tests':
      case 'examples':
        return type
      default:
    }
    return super._getIdForElement(el, type)
  }
}
