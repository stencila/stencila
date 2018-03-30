import { HttpStorageClient, VfsStorageClient, InMemoryDarBuffer } from 'substance'
import { WebAppChrome } from 'substance-texture'
import StencilaArchive from './StencilaArchive'

import {
  _renderStencilaApp,
  _setupStencilaChildContext,
  _initStencilaContext,
  _initStencilaArchive
} from './stencilaAppHelpers'

export default class StencilaWebApp extends WebAppChrome {

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
    let storage
    if (this.props.storageType==='vfs') {
      storage = new VfsStorageClient(window.vfs, './examples/')
    } else {
      storage = new HttpStorageClient(this.props.storageUrl)
    }
    let buffer = new InMemoryDarBuffer()
    let archive = new StencilaArchive(storage, buffer, context)
    return archive.load(archiveId)
  }

  _initArchive(archive, context) {
    return _initStencilaArchive(archive, context)
  }
}
