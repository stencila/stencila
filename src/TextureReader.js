import {
  getQueryStringParam, substanceGlobals, platform, Component
} from 'substance'
import { vfsSaveHook } from 'substance-texture'
import StencilaArchive from './StencilaArchive'
import StencilaWebApp from './StencilaWebApp'

substanceGlobals.DEBUG_RENDERING = platform.devtools

// This uses a monkey-patched VfsStorageClient that checks immediately
// if the stored data could be loaded again, or if there is a bug in
// Textures exporter
class DevWebApp extends StencilaWebApp {
  _getStorage(storageType) {
    let storage = super._getStorage(storageType)
    if (storageType === 'vfs') {
      vfsSaveHook(storage, StencilaArchive)
    }
    return storage
  }
}

/*
  Component that can be embedded in journal pages.

  RDSReader.mount({
    archiveId: 'e24351',
    storageUrl: 'https://dar-archives.elifesciences.org'
  })
*/
export default class TextureReader extends Component {
  render($$) {
    let el = $$('div').addClass('sc-texture-reader')
    el.append(
      $$(DevWebApp, {
        archiveId: this.props.archiveId,
        storageType: this.props.storageUrl ? 'fs' : 'vfs',
        storageUrl: this.props.storageUrl
      })
    )
    return el
  }
}

window.TextureReader = TextureReader


