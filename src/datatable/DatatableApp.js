import Component from 'substance/ui/Component'
import DocumentSession from 'substance/model/DocumentSession'

import Datatable from './Datatable'
import DatatableConfigurator from './DatatableConfigurator'
// import GridEditor from './editors/grid/GridEditor'
// import CodeEditorComponent from '../document/ui/CodeEditorComponent'
import code from '../utilities/code'

class DatatableApp extends Component {

  getInitialState () {
    return {
    }
  }

  render ($$) {
    var el = $$('div').addClass('sc-datatable-app')

    if (this.state.documentSession) {
      // var editorProps = {
      //   // Other required props
      //   documentSession: this.state.documentSession,
      //   configurator: new DatatableConfigurator()
      // }
      // el.append($$(GridEditor, editorProps).ref('gridEditor'))

      el.append(
        $$('input').ref('query')
      )
    } else {
      el
        .addClass('sm-loading')
        .append(
          $$('i')
            .addClass('fa fa-spinner fa-pulse fa-fw')
        )
    }

    return el
  }

  didMount () {
    // Load initial datatable
    this.loadDatatable(this.props.data, this.props.format)
    // Load ACE editor
    code.loadAce()
  }

  /**
   * Load a new datatable into the application
   *
   * This may be called for example when a query is made
   * to a datatable and the result datatable needs to be loaded
   *
   * @param      {<type>}  data  The data
   * @param      {string}  format   The format
   */
  loadDatatable (data, format) {
    var doc
    var documentSession

    if (format === 'html') {
      // Import 
      // var htmlImporter = new DatatableHTMLImporter({
      //   configurator: new DatatableConfigurator()
      // })
      // doc = htmlImporter.importDocument(this.props.data)
      doc = new Datatable()
      // Create a new local document session
      documentSession = new DocumentSession(doc)
      // For code compatability with a `CollabSession` ...
      documentSession.config = {
        user: null,
        rights: null
      }
    } else if (format === 'json') {
      throw Error('Not yet implemented')
    } else {
      throw Error('Not yet implemented')
    }

    // Extend state to trigger rerendering
    this.extendState({
      doc: doc,
      documentSession: documentSession
    })
  }

}

export default DatatableApp
