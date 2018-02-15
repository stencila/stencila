import { Component, DefaultDOMElement, platform } from 'substance'
import { EditorPackage as TextureEditorPackage } from 'substance-texture'
import SheetEditor from '../sheet/SheetEditor'
import ProjectBar from './ProjectBar'
import HelpComponent from '../shared/HelpComponent'
import HostsComponent from '../host/HostsComponent'

export default class Project extends Component {

  didMount() {
    this.handleActions({
      'openDocument': this._openDocument,
      'openHelp': this._openHelp,
      'toggleHelp': this._toggleHelp,
      'toggleHosts': this._toggleHosts
    })

    if (platform.inBrowser) {
      this.documentEl = DefaultDOMElement.wrapNativeElement(document)
      this.documentEl.on('keydown', this.onKeyDown, this)
    }
  }

  _dispose() {
    if (platform.inBrowser) {
      this.documentEl.off(this)
    }
  }

  getInitialState() {
    let activeDocument = this._getActiveDocument()
    return {
      contextId: undefined,
      contextProps: undefined,
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
      urlResolver: this.props.documentArchive
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-project')
    el.append(
      $$('div').addClass('se-main-pane').append(
        this._renderEditorPane($$)
      ),
      $$(ProjectBar, {
        contextId: this.state.contextId,
        documentId: this.state.documentId,
        documentArchive: this.props.documentArchive
      })
    )
    return el
  }

  _getPubMetaDbSession() {
    return this._getDocumentArchive().getEditorSession('pub-meta')
  }

  _getActiveDocument() {
    let archive = this._getDocumentArchive()
    let firstEntry = archive.getDocumentEntries()[0]
    return firstEntry
  }

  _getActiveEditorSession() {
    let documentId = this.state.documentId
    return this.props.documentArchive.getEditorSession(documentId)
  }

  _getDocumentArchive() {
    return this.props.documentArchive
  }

  _getDocumentRecordById(id) {
    let dc = this._getDocumentArchive()
    let entries = dc.getDocumentEntries()
    return entries.find(e => e.id === id)
  }

  _renderEditorPane($$) {
    let el = $$('div').addClass('se-editor-pane')
    let documentId = this.state.documentId
    let documentRecord = this._getDocumentRecordById(documentId)
    let documentType = documentRecord.type
    let da = this._getDocumentArchive()
    let editorSession = da.getEditorSession(documentId)
    let contextComponent = this._getContextComponent($$)

    if (documentType === 'application/jats4m') {
      el.append(
        $$(TextureEditorPackage.Editor, {
          editorSession,
          pubMetaDbSession: this._getPubMetaDbSession(),
          contextComponent
        }).ref('editor')
          .addClass('sc-article-editor')
      )
    } else if (documentType === 'application/sheetml') {
      el.append(
        $$(SheetEditor, {
          editorSession,
          contextComponent
        }).ref('editor')
      )
    }
    return el
  }

  _openDocument(documentId) {
    this.extendState({
      documentId: documentId
    })
  }

  /*
    E.g. _openHelp('function/sum')
  */
  _openHelp(page) {
    this.extendState({
      contextId: 'help',
      contextProps: {
        page
      }
    })
  }

  /*
    Either hide help or show function index
  */
  _toggleHelp() {
    if (this.state.contextId === 'help') {
      this.extendState({
        contextId: undefined,
        contextProps: undefined
      })
    } else {
      this.extendState({
        contextId: 'help',
        contextProps: { page: 'function/index'}
      })
    }
  }

  /*
    Either open or hide hosts connection information
  */
  _toggleHosts() {
    if (this.state.contextId === 'hosts') {
      this.extendState({
        contextId: undefined,
        contextProps: undefined
      })
    } else {
      this.extendState({
        contextId: 'hosts',
        contextProps: { page: 'hosts'}
      })
    }
  }

  onKeyDown(event) {
    // ignore fake IME events (emitted in IE and Chromium)
    if ( event.key === 'Dead' ) return
    // Handle custom keyboard shortcuts globally
    let editorSession = this._getActiveEditorSession()
    let custom = editorSession.keyboardManager.onKeydown(event)
    return custom
  }

  /*
    TODO: We may want to make this extensible in the future
  */
  _getContextComponent($$) {
    let contextId = this.state.contextId
    let contextProps = this.state.contextProps
    let contextComponent
    if (contextId === 'help') {
      contextComponent = $$(HelpComponent, contextProps).ref('contextComponent')
    } else if (contextId === 'hosts') {
      contextComponent = $$(HostsComponent, contextProps).ref('contextComponent')
    } else if (contextId === 'issues') {
      console.warn('TODO: use issue component')
    }
    return contextComponent
  }

}
