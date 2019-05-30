import { Component, DefaultDOMElement, platform } from 'substance'
import { EditorPackage as TextureEditorPackage } from 'substance-texture'
import SheetEditor from '../sheet/SheetEditor'
import ContextPane from './ContextPane'
import { addNewDocument } from './ProjectManipulations'
import _initStencilaArchive from '../shared/_initStencilaArchive'

export default class Project extends Component {

  constructor(...args) {
    super(...args)

    // Store the viewports, so we can restore scroll positions
    this._viewports = {}

    this.appState = {
      reproduce: false,
      engineRunning: false
    }
  }

  didMount() {
    this.handleActions({
      'addDocument': this._addDocument,
      'openDocument': this._openDocument,
      'removeDocument': this._removeDocument,
      'updateDocumentName': this._updateDocumentName,
      'closeContext': this._closeContext,
      'openHelp': this._openHelp,
      'toggleHelp': this._toggleHelp,
      'toggleHosts': this._toggleHosts,
      'toggleReproduce': this._toggleReproduce
    })

    if (platform.inBrowser) {
      this.documentEl = DefaultDOMElement.wrapNativeElement(document)
      this.documentEl.on('keydown', this.onKeyDown, this)
    }

    // HACK: we enable reproduce mode by default
    this._toggleReproduce()
  }

  willUpdateState() {
    let oldDocumentId = this.state.documentId
    this._viewports[oldDocumentId] = this.refs.editor.getViewport()
  }

  _dispose() {
    if (platform.inBrowser) {
      this.documentEl.off(this)
    }
  }

  getInitialState() {
    let activeDocument = this._getActiveDocument()
    return {
      documentId: activeDocument.id
    }
  }

  getChildContext() {
    // ATTENTION: we should be careful with adding things here.
    // If something is missing, we likely should fix it somewhere else.
    // Add only project related things here.
    // One example for what better not to add: 'pubMetaDbSession'.
    // This is passed to Texture as prop which in turn exposes it via childContext.
    return {
      documentArchive: this.props.documentArchive,
      urlResolver: this.props.documentArchive,
      appState: this.appState
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-project')
    el.append(
      $$('div').addClass('se-main-pane').append(
        this._renderEditorPane($$),
        $$(ContextPane, {
          contextId: this._contextId,
          contextProps: this._contextProps
        }).ref('contextPane')
      )
      // $$(ProjectBar, {
      //   contextId: this._contextId,
      //   documentId: this.state.documentId,
      //   archive: this.props.documentArchive
      // }).ref('projectBar')
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
    let viewport = this._viewports[documentId]
    let da = this._getDocumentArchive()
    let editorSession = da.getEditorSession(documentId)

    if (documentType === 'article') {
      el.append(
        $$(TextureEditorPackage.Editor, {
          viewport,
          editorSession,
          pubMetaDbSession: this._getPubMetaDbSession(),
          disabled: true
        }).ref('editor')
          .addClass('sc-article-editor')
      )
    } else if (documentType === 'sheet') {
      el.append(
        $$(SheetEditor, {
          viewport,
          editorSession
        }).ref('editor')
      )
    }
    return el
  }

  _addDocument(type) {
    let archive = this._getDocumentArchive()
    let newDocumentId = addNewDocument(archive, type)
    this._openDocument(newDocumentId)
  }

  _openDocument(documentId) {
    this.extendState({
      documentId: documentId
    })
  }

  _updateDocumentName(documentId, name) { // eslint-disable-line no-unused-vars
    let archive = this._getDocumentArchive()
    archive.renameDocument(documentId, name)
    this.refs.projectBar.rerender()
  }

  _removeDocument(documentId) { // eslint-disable-line no-unused-vars
    let archive = this._getDocumentArchive()
    let documentEntries = archive.getDocumentEntries()
    if (documentEntries.length > 1) {
      archive.removeDocument(documentId)
      let firstDocument = this._getActiveDocument()
      this.extendState({
        documentId: firstDocument.id
      })
    } else {
      console.warn('Not allowed to delete the last document in the archive. Skipping.')
    }
  }

  /*
    E.g. _openHelp('function/sum')
  */
  _openHelp(page) {
    this._contextId = 'help'
    this._contextProps = { page }
    this.refs.contextPane.extendProps({
      contextId: this._contextId,
      contextProps: this._contextProps
    })
    this.refs.projectBar.extendProps({
      contextId: this._contextId
    })
  }

  _closeContext() {
    this._contextId = undefined
    this._contextProps = undefined
    this.refs.contextPane.extendProps({
      contextId: this._contextId,
      contextProps: this._contextProps
    })
    this.refs.projectBar.extendProps({
      contextId: this._contextId
    })
  }

  /*
    Either hide help or show function index
  */
  _toggleHelp() {
    let contextId = this._contextId
    if (contextId === 'help') {
      this._contextId = undefined
      this._contextProps = undefined
    } else {
      this._contextId = 'help'
      this._contextProps = { page: 'function/index'}
    }
    this.refs.contextPane.extendProps({
      contextId: this._contextId,
      contextProps: this._contextProps
    })
    this.refs.projectBar.extendProps({
      contextId: this._contextId
    })
  }

  _toggleReproduce () {
    // TODO: we should update the state after the engine has been
    // started successfully
    let reproduce = !this.appState.reproduce
    this.appState.reproduce = reproduce
    if (reproduce && !this.appState.engineRunning) {
      this._launchExecutionEngine().then(running => {
        if (!running) {
          this._toggleReproduce()
        }
      })
    }
    this._updateCellComponents()
  }

  _launchExecutionEngine () {
    return new Promise((resolve) => {
      // resolve(window.confirm('Start the Engine?'))
      resolve(true)
    }).then(yesPlease => {
      if (yesPlease) {
        const archive = this.props.documentArchive
        return _initStencilaArchive(archive, this.context).then(() => {
          this.appState.engineRunning = true
          return true
        })
      }
      return false
    })
  }

  _updateCellComponents () {
    // Update all cell nodes in the document
    let cellComps = this.findAll('.sc-cell')
    cellComps.forEach((cellComponent) => {
      cellComponent.extendState({
        hideCodeToggle: !this.appState.reproduce,
        hideCode: true
      })
    })
  }

  /*
    Either open or hide hosts connection information
  */
  _toggleHosts() {
    let contextId = this._contextId
    if (contextId === 'hosts') {
      this._contextId = undefined
      this._contextProps = undefined
    } else {
      this._contextId = 'hosts'
      this._contextProps = { page: 'hosts' }
    }

    this.refs.contextPane.extendProps({
      contextId: this._contextId,
      contextProps: this._contextProps
    })
    this.refs.projectBar.extendProps({
      contextId: this._contextId
    })
  }

  onKeyDown(event) {
    // ignore fake IME events (emitted in IE and Chromium)
    if ( event.key === 'Dead' ) return
    // Handle custom keyboard shortcuts globally
    let editorSession = this._getActiveEditorSession()
    let custom = editorSession.keyboardManager.onKeydown(event)
    return custom
  }

}
