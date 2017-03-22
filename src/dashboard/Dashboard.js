import { Component, FontAwesomeIcon, BrowserDOMElement } from 'substance'
import timeago from 'timeago.js'

export default class Dashboard extends Component {

  didMount() {
    this._fetchDocuments()
  }

  _fetchDocuments() {
    let backend = this.getBackend()
    backend.listDocuments().then((documents) => {
      this.setState({
        documents: documents
      })
    })
  }

  /*
    Explicit reload of the dashboard
  */
  reload() {
    this._fetchDocuments()
  }

  getBackend() {
    return this.props.backend
  }

  render ($$) {
    let el = $$('div').addClass('sc-dashboard')
    let resolveEditorURL = this.props.resolveEditorURL
    let documents = this.state.documents
    if (documents) {
      documents.forEach((doc) => {
        let docTypeIcon = doc.type === 'document' ? 'fa-file-text' : 'fa-table';
        el.append(
          $$('div').addClass('se-document-entry').append(
            $$('div').addClass('se-title').append(
              $$('a')
              .attr('href', resolveEditorURL(doc.type, doc.id))
              .attr('target', '_blank')
              .append(
                $$(FontAwesomeIcon, {icon: docTypeIcon }),
                ' ',
                doc.title
              )
            ),
            this._renderMeta($$, doc),
            $$('div').addClass('se-actions-dropdown').append(
              $$('button').addClass('se-actions-toggle').append(
                $$(FontAwesomeIcon, {icon: 'fa-ellipsis-v' })
              ).on('click', this._toggleActions),
              this._renderActions($$, doc)
            )
          )
        )
      })
    }
    return el
  }

  _renderMeta($$, doc) {
    let el = $$('div').addClass('se-meta')
    // Only display file path for docs with external storage
    if (doc.storage.external) {
      el.append(
        $$('div').addClass('se-file-path').append(
          [doc.storage.folderPath, doc.storage.fileName].join('/')
        )
      )
    }
    el.append(
      $$('div').addClass('se-updated-at').append(
        'updated ',
        timeago().format(new Date(doc.updatedAt))
      )
    )
    return el
  }

  _renderActions($$, doc) {
    let el = $$('div').addClass('se-actions')

    if (doc.storage.external) {
      el.append(
        $$('button').addClass('se-action')
          .append('Unlink')
          .on('click', this._deleteDocument.bind(this, doc.id))
      )
    } else {
      el.append(
        $$('button').addClass('se-action')
          .append('Delete')
          .on('click', this._deleteDocument.bind(this, doc.id))
      )
    }
    return el
  }

  _deleteDocument(documentId) {
    let backend = this.getBackend()
    backend.deleteDocument(documentId).then(() => {
      this.reload()
    })
  }

  _toggleActions(e) {
    let targetEl = BrowserDOMElement.wrapNativeElement(e.currentTarget)
    let actionDropDownEl = targetEl.getParent()
    if (actionDropDownEl.hasClass('sm-expanded')) {
      actionDropDownEl.removeClass('sm-expanded')
    } else {
      actionDropDownEl.addClass('sm-expanded')
    }
  }

}
