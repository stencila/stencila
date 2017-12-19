import { Component } from 'substance'
import { EditorPackage as TextureEditorPackage } from 'substance-texture'
import SheetEditor from '../sheet/SheetEditor'

import ProjectBar from './ProjectBar'

export default class Project extends Component {

  didMount() {
    this.handleActions({
      'openDocument': this._openDocument
    })
  }

  getInitialState() {
    let activeDocument = this._getActiveDocument()
    return {
      documentId: activeDocument.id
    }
  }

  getChildContext() {
    let pubMetaDbSession = this._getPubMetaDbSession()
    return {
      functionManager: this.props.functionManager,
      cellEngine: this.props.engine,
      host: this.props.host,
      pubMetaDbSession: pubMetaDbSession,
      // LEGACY:
      get db() {
        console.warn('DEPRECATED: Use context.pubMetaDbSession.getDocument()')
        return pubMetaDbSession.getDocument()
      },
      get dbSession() {
        console.warn('DEPRECATED: Use context.pubMetaDbSession')
        return pubMetaDbSession
      },
      get entityDbSession() {
        console.warn('DEPRECATED: Use context.pubMetaDbSession')
        return pubMetaDbSession
      }
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-project')
    el.append(
      $$('div').addClass('se-main-pane').append(
        this._renderEditorPane($$)
      ),
      $$(ProjectBar, {
        documentId: this.state.documentId,
        documentContainer: this.props.documentContainer
      })
    )
    return el
  }

  _getPubMetaDbSession() {
    return this._getDocumentContainer().getEditorSession('pub-meta')
  }

  _getActiveDocument() {
    let dc = this._getDocumentContainer()
    return dc.getDocumentEntries()[0]
  }

  _getDocumentContainer() {
    return this.props.documentContainer
  }

  _getDocumentRecordById(id) {
    let dc = this._getDocumentContainer()
    return dc.getDocumentEntries().find(e => e.id === id)
  }

  _renderEditorPane($$) {
    let el = $$('div').addClass('se-editor-pane')
    let documentId = this.state.documentId
    let documentRecord = this._getDocumentRecordById(documentId)
    let documentType = documentRecord.type
    let dc = this._getDocumentContainer()
    let editorSession = dc.getEditorSession(documentId)

    if (documentType === 'article') {
      el.append(
        $$(TextureEditorPackage.Editor, {
          editorSession,
          entityDbSession: this._getPubMetaDbSession()
        })
      )
    } else if (documentType === 'sheet') {
      el.append(
        $$(SheetEditor, {
          editorSession
        })
      )
    }
    return el
  }

  _openDocument(documentId) {
    this.setState({
      documentId: documentId
    })
  }

}
