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
  StencilaWeb.mount({
    archiveId: getQueryStringParam('archive') || 'kitchen-sink',
    storageType: getQueryStringParam('storage') || 'vfs',
    storageUrl: getQueryStringParam('storageUrl') || '/archives'
  }, window.document.body)
})


class AppChrome extends Component {

  didMount() {
    this._init()
    DefaultDOMElement.getBrowserWindow().on('keydown', this._keyDown, this)
    DefaultDOMElement.getBrowserWindow().on('drop', this._supressDnD, this)
    DefaultDOMElement.getBrowserWindow().on('dragover', this._supressDnD, this)
  }

  dispose() {
    DefaultDOMElement.getBrowserWindow().off(this)
  }

  getChildContext() {
    return this._childContext
  }

  getInitialState() {
    return {
      archive: undefined,
      error: undefined
    }
  }

  /*
    4 initialisation stages

    - _setupChildContext (sync)
    - _initContext (async)
    - _loadArchive (async)
    - _initArchive (async)
  */
  _init() {
    this._childContext = this._setupChildContext()

    let promise = this._initContext(this._childContext)
    .then(() => this._loadArchive(this.props.archiveId, this._childContext))
    .then(archive => this._initArchive(archive, this._childContext))
    .then(archive => {
      this.setState({archive})
    })

    if (!platform.devtools) {
      promise.catch(error => {
        console.error(error)
        this.setState({error})
      })
    }
  }

  _setupChildContext() {
    throw new Error('_setupChildContext not implemented')
  }

  _initContext(context) {
    return Promise.resolve(context)
  }

  _loadArchive() {
    throw new Error('_loadArchive not implemented')
  }

  _initArchive() {
    throw new Error('_initArchive not implemented')
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

  _keyDown(event) {
    if ( event.key === 'Dead' ) return
    if (this._handleKeyDown) {
      this._handleKeyDown(event)
    }
  }

  _supressDnD(event) {
    event.preventDefault()
  }

}

class WebAppChrome extends AppChrome {
  _handleKeyDown(event) {
    let key = parseKeyEvent(event)
    // CommandOrControl+S
    if (key === 'META+83' || key === 'CTRL+83') {
      this._save()
      event.preventDefault()
    }
  }
}

export default class StencilaWeb extends WebAppChrome {

  render($$) {
    return _renderStencilaApp($$, this)
  }

  _setupChildContext() {
    return _setupStencilaChildContext(this.context)
  }

  _initContext(context) {
    return _initStencilaContext(context)
  }

  _loadArchive(archiveId, context) {
    return _loadStencilaArchive(
      archiveId,
      context,
      this.props.storageType,
      this.props.storageUrl
    )
  }

  _initArchive(archive, context) {
    return _initStencilaArchive(archive, context)
  }
}


function _renderStencilaApp($$, app) {
  let el = $$('div').addClass('sc-app')
  let { archive, error } = app.state
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


function _setupStencilaChildContext(originalContext) {
  const context = setupStencilaContext()
  return Object.assign({}, originalContext, context)
}

function _initStencilaContext(context) {
  return context.host.initialize()
}

function _loadStencilaArchive(archiveId, context, storageType, storageUrl) {
  let storage
  if (storageType==='vfs') {
    storage = new VfsStorageClient(window.vfs, './examples/')
  } else {
    storage = new HttpStorageClient(storageUrl)
  }
  let buffer = new InMemoryDarBuffer()
  let archive = new StencilaArchive(storage, buffer, context)
  return archive.load(archiveId)
}


function _initStencilaArchive(archive, {engine}) {
  // register documents and sheets with the engine
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
  // start the engine
  const ENGINE_REFRESH_INTERVAL = 10 // ms
  engine.run(ENGINE_REFRESH_INTERVAL)
  return Promise.resolve(archive)
}
