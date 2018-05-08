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

  _getArchiveClass() {
    return StencilaArchive
  }

  _initArchive(archive, context) {
    return _initStencilaArchive(archive, context)
  }
}
