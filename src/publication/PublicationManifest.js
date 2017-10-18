import { DefaultDOMElement } from 'substance'


export default class PublicationManifest {
  constructor(manifestXML) {
    this.manifestDoc = DefaultDOMElement.parseXML(manifestXML)
  }

  getDocuments() {
    return this.manifestDoc.findAll('document').map((doc) => {
      return {
        path: doc.attr('src'),
        type: doc.attr('type'),
        name: doc.textContent
      }
    })
  }
}
