import { InMemoryDarBuffer } from 'substance'
import { DesktopAppChrome } from 'substance-texture'
import StencilaArchive from './StencilaArchive'

import {
  _renderStencilaApp,
  _setupStencilaChildContext,
  _initStencilaContext,
  _initStencilaArchive
} from './stencilaAppHelpers'

export default class StencilaDesktopApp extends DesktopAppChrome {

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
    let storage = new this.props.FSStorageClient()
    let buffer = new InMemoryDarBuffer()
    let archive = new StencilaArchive(storage, buffer, context)
    // HACK: this should be done earlier in the lifecycle (after first didMount)
    // and later disposed properly. However we can accept this for now as
    // the app lives as a singleton atm.
    // NOTE: _archiveChanged is implemented by DesktopAppChrome
    archive.on('archive:changed', this._archiveChanged, this)
    return archive.load(archiveId)
  }

  _initArchive(archive, context) {
    return _initStencilaArchive(archive, context)
  }
}
