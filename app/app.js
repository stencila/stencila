import {
  getQueryStringParam, Component, DefaultDOMElement, parseKeyEvent,
  HttpStorageClient, VfsStorageClient, InMemoryDarBuffer, substanceGlobals,
  platform, forEach
} from 'substance'
import { JATSImportDialog } from 'substance-texture'
import {
  Project,
  setupStencilaContext,
  StencilaArchive,
  SheetAdapter, ArticleAdapter
} from 'stencila'

window.addEventListener('load', () => {
  substanceGlobals.DEBUG_RENDERING = platform.devtools
  App.mount({}, window.document.body)
})

class App extends Component {

  constructor(...args) {
    super(...args)

    // this is initialized in _init()
    this._childContext = null
  }

  getInitialState() {
    return {
      archive: undefined,
      error: undefined
    }
  }

  getChildContext() {
    return this._childContext
  }

  didMount() {
    this._init()
    DefaultDOMElement.getBrowserWindow().on('keydown', this._keyDown, this)
  }

  dispose() {
    DefaultDOMElement.getBrowserWindow().off(this)
  }

  render($$) {
    let el = $$('div').addClass('sc-app')
    let { archive, error } = this.state
    if (archive) {
      el.append(
        $$(Project, {
          documentArchive: archive
        })
      )
    } else if (error) {
      if (error.type === 'jats-import-error') {
        el.append(
          $$(JATSImportDialog, { errors: error.detail })
        )
      } else {
        el.append(
          'ERROR:',
          error.message
        )
      }
    } else {
      // LOADING...
    }
    return el
  }

  _init() {
    let archiveId = getQueryStringParam('archive') || 'kitchen-sink'
    const context = setupStencilaContext()
    const { host, engine } = context
    // update the component's context which is
    this._childContext = Object.assign({}, this.context, context)

    // initialize the host
    host.initialize()
    // load the archive
    .then(() => this._loadArchive(archiveId, context))
    .then(archive => {
      // register documents and sheets with the engine
      this._connectArchiveEntriesWithEngine(archive, engine)
      // start the engine
      const ENGINE_REFRESH_INTERVAL = 10 // ms
      engine.run(ENGINE_REFRESH_INTERVAL)
      // finally trigger a rerender with the loaded article
      this.setState({ archive })
    })
    // .catch(error => {
    //   console.error(error)
    //   this.setState({error})
    // })
  }

  _loadArchive(archiveId, context) {
    let storageType = getQueryStringParam('storage') || 'vfs'
    let storageUrl = getQueryStringParam('storageUrl') || '/archives'
    let storage
    if (storageType==='vfs') {
      storage = new VfsStorageClient(window.vfs, './examples/')
    } else {
      storage = new HttpStorageClient(storageUrl)
    }
    const buffer = new InMemoryDarBuffer()
    const archive = new StencilaArchive(storage, buffer, context)
    return archive.load(archiveId)
  }

  _connectArchiveEntriesWithEngine(archive, engine) {
    let entries = archive.getDocumentEntries()
    forEach(entries, entry => {
      let { id, type } = entry
      let editorSession = archive.getEditorSession(id)
      let Adapter
      if (type === 'article') {
        Adapter = ArticleAdapter
      } else if (type === 'sheet') {
        Adapter = SheetAdapter
      }
      if (Adapter) {
        Adapter.connect(engine, editorSession, id)
      }
    })
  }

  /*
    We may want an explicit save button, that can be configured on app level,
    but passed down to editor toolbars.
  */
  _save() {
    this.state.archive.save().then(() => {
      console.info('successfully saved')
    }).catch(err => {
      console.error(err)
    })
  }

  _keyDown(e) {
    let key = parseKeyEvent(e)
    // CommandOrControl+S
    if (key === 'META+83' || key === 'CTRL+83') {
      this._save()
      e.preventDefault()
    }
  }
}
