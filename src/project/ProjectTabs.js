import { Component } from 'substance'

export default class ProjectTabs extends Component {

  render($$) {
    let el = $$('div').addClass('sc-project-tabs')
    let dc = this.props.documentContainer
    let documentEntries = dc.getDocumentEntries()

    documentEntries.forEach((entry) => {
      let button = $$('button').append(entry.name)
        .on('click', this._openDocument.bind(this, entry.id))
      if (this.props.documentId === entry.id) {
        button.addClass('sm-active')
      }
      el.push(button)
    })
    return el
  }

  _openDocument(documentId) {
    this.send('openDocument', documentId)
  }

}
