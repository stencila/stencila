import { Component, FontAwesomeIcon, BrowserDOMElement } from 'substance'

export default class Dashboard extends Component {

  getBackend() {
    return this.props.backend
  }

  didMount() {
    let backend = this.getBackend()
    // Retrieve document records from the backend
    backend.listDocuments().then((documents) => {
      this.setState({
        documents: documents
      })
    })
  }

  render ($$) {
    let el = $$('div').addClass('sc-dashboard')

    let documents = this.state.documents
    if (documents) {
      documents.forEach((doc) => {
        let docTypeIcon = doc.type === 'document' ? 'fa-file-text' : 'fa-table';
        el.append(
          $$('div').addClass('se-document-entry').append(
            // $$('div').addClass('se-icon').append(
            //
            // ),
            $$('div').addClass('se-title').append(
              $$('a').attr('href', '#').append(
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
