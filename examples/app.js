import {
  getQueryStringParam, Component, DefaultDOMElement, parseKeyEvent,
  HttpStorageClient, VfsStorageClient, InMemoryDarBuffer, substanceGlobals,
  platform
} from 'substance'

import {
  Project,
  setupStencilaContext,
  StencilaArchive
} from 'stencila'

window.addEventListener('load', () => {
  substanceGlobals.DEBUG_RENDERING = platform.devtools
  App.mount({}, window.document.body)
})

import iceCreamSales from './util/ice-cream-sales'

// prepare the VFS on-the-fly expanding all examples
let vfs = window.vfs
// Add a sheet to dar dynamically
vfs.writeFileSync('examples/data/publication/ice-cream-sales.xml', iceCreamSales().innerHTML)

class App extends Component {

  didMount() {
    this._init()
    DefaultDOMElement.getBrowserWindow().on('keydown', this._keyDown, this)
  }

  dispose() {
    DefaultDOMElement.getBrowserWindow().off(this)
  }

  getInitialState() {
    return {
      archive: undefined,
      error: undefined
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-app')
    let { archive, host, functionManager, engine } = this.state

    if (archive) {

      el.append(
        $$(Project, {
          documentArchive: archive,
          host,
          functionManager,
          engine
        })
      )
    } else {
      // LOADING...
    }
    return el
  }

  _init() {
    let archiveId = getQueryStringParam('archive') || 'publication'
    let storageType = getQueryStringParam('storage') || 'vfs'
    let storageUrl = getQueryStringParam('storageUrl') || '/archives'
    let storage
    if (storageType==='vfs') {
      storage = new VfsStorageClient(window.vfs, './examples/data/')
    } else {
      storage = new HttpStorageClient(storageUrl)
    }
    let buffer = new InMemoryDarBuffer()
    let archive = new StencilaArchive(storage, buffer)
    archive.load('examples/data/'+archiveId)
    .then(() => {
      return setupStencilaContext(archive)
    }).then(({host, functionManager, engine}) => {
      this.setState({archive, functionManager, engine, host})
    })
    // .catch(error => {
    //   console.error(error)
    //   this.setState({error})
    // })
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
    if (key === 'META+83') {
      this._save()
      e.preventDefault()
    }
  }
}
