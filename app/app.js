import {
  getQueryStringParam, substanceGlobals, platform
} from 'substance'
import { vfsSaveHook } from 'substance-texture'
import { StencilaWebApp, StencilaArchive } from 'stencila'

window.addEventListener('load', () => {
  substanceGlobals.DEBUG_RENDERING = platform.devtools
  DevWebApp.mount({
    archiveId: getQueryStringParam('archive') || 'kitchen-sink',
    storageType: getQueryStringParam('storage') || 'vfs',
    storageUrl: getQueryStringParam('storageUrl') || '/archives'
  }, window.document.body)
})

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