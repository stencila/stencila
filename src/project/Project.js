import { Component } from 'substance'
import { EditorPackage as TextureEditorPackage } from 'substance-texture'
import SheetEditor from '../sheet/SheetEditor'

// import ProjectBar from './ProjectBar'

export default class Project extends Component {

  getInitialState() {
    let activeDocument = this._getActiveDocument()
    return {
      documentId: activeDocument.id,
      documentType: activeDocument.type
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
      )//,
      // $$(ProjectBar, {
      //   documentId: this.state.documentId,
      //   documentContainer: this.props.documentContainer
      // })
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

  _renderEditorPane($$) {
    let el = $$('div').addClass('se-editor-pane')
    let documentType = this.state.documentType
    let documentId = this.state.documentId
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

}
