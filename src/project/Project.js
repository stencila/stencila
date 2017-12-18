import { Component } from 'substance'
import { EditorPackage as TextureEditorPackage } from 'substance-texture'
import SheetEditor from '../sheet/SheetEditor'

import ProjectBar from './ProjectBar'

export default class Project extends Component {

  getInitialState() {
    let activeDocument = this._getActiveDocument()
    return {
      documentId: activeDocument.id,
      documentType: activeDocument.type
    }
  }

  getChildContext() {
    return {
      functionManager: this.props.functionManager,
      cellEngine: this.props.engine,
      host: this.props.engine.getHost(),
      // TODO: use a more specific name (e.g. pubMetaDb)
      pubMetaDbSession: this._getPubMetaDbSession()
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
    return this.documentContainer.getEditorSession('pub-meta')
  }

  _getActiveDocument() {
    let dc = this.props.documentContainer
    return dc.getDocumentEntries()[0]
  }

  _getDocumentContainer() {
    return this.props.documentContainer
  }

  _renderEditorPane($$) {
    let el = $$('div').addClass('se-editor-pane')
    let activeDocumentType = this.state.type
    let activeDocumentId = this.state.type
    let dc = this._getDocumentContainer()
    let editorSession = dc.getEditorSession(activeDocumentId)

    if (activeDocumentType === 'article') {
      el.append(
        $$(TextureEditorPackage.Editor, {
          editorSession
        })
      )
    } else if (activeDocumentType === 'sheet') {
      el.append(
        $$(SheetEditor, {
          editorSession
        })
      )
    }
    return el
  }

}
