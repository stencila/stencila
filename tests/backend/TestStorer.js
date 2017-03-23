import { MemoryBuffer } from '../../index.es'

export default class TestStorer extends MemoryBuffer {
  constructor (archivePath, mainFilePath, isExternal) {
    super()
    this.archivePath = archivePath
    this.mainFilePath = mainFilePath
    this._isExternal = isExternal
  }

  getArchivePath() {
    return this.archivePath
  }

  getMainFilePath() {
    return this.mainFilePath
  }

  isExternal() {
    return this._isExternal
  }

  getType() {
    return 'filesystem'
  }
}
