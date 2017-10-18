import MemoryBuffer from './MemoryBuffer'

export default class MemoryBackend {
  /*
    Takes an object with documentIds and HTML content
  */
  constructor(vfs) {
    this.vfs = vfs
  }

  /*
    Returns a buffer object.

    Use MemoryBuffer implementation as an API reference
  */
  getBuffer(publicationId) {
    let buffer = new MemoryBuffer(this.vfs, `data/${publicationId}`)
    return Promise.resolve(buffer)
  }

  storeBuffer(/*buffer*/) {
    return Promise.resolve()
  }

  updateManifest(/* documentId, props */) {
    return Promise.resolve()
  }

}
