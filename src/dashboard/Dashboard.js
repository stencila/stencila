import { Component } from 'substance'

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
        el.append(
          $$('div')
            .addClass('se-document-entry')
            .append(
              $$('div').addClass('se-title').append(
                $$('a').attr('href', '#').append(doc.title)
              ),
              $$('div').addClass('se-address').append(
                doc.address
              )
            )
        )
      })
    }
    return el
  }

}
