/* eslint-disable no-unused-vars */
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

  _getDefaultDataFolder() {
    return './examples/'
  }

  _initArchive(archive, context) {
    // return _initStencilaArchive(archive, context)
    // HACK: do not connect the archive with the engine right away
    // we gonna do this when the user asks to switch to reproducible mode
    return Promise.resolve(archive)
  }
}
