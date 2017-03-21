import { Component, FontAwesomeIcon, BrowserDOMElement } from 'substance'

export default class Dashboard extends Component {

  didMount() {
    this._fetchDocuments()
  }

  _fetchDocuments() {
    console.log('fetching docs...')
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
              .attr('href', resolveEditorURL(doc.type, doc.address))
              .attr('target', '_blank')
              .append(
                $$(FontAwesomeIcon, {icon: docTypeIcon }),
                ' ',
                doc.title
              )
            ),
            $$('div').addClass('se-address').append(
              doc.address
            ),
            $$('div').addClass('se-actions-dropdown').append(
              $$('button').addClass('se-actions-toggle').append(
                $$(FontAwesomeIcon, {icon: 'fa-ellipsis-v' })
              ).on('click', this._toggleActions),
              $$('div').addClass('se-actions').append(
                $$('button').addClass('se-action').append('Delete'),
                $$('button').addClass('se-action').append('Open')
              )
            )
          )
        )
      })
    }
    return el
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
