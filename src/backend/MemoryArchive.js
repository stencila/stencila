/* globals Blob */

/*
  In-memory archive (cmp. mini filesytem) for representing Substance documents with assets
*/
export default class MemoryArchive {
  constructor() {
    this._files = {}
  }

  /*
    File data must either be a utf8 string or a blob object
  */
  writeFile(path, mimeType, data) {
    return new Promise((resolve, reject) => {
      if (typeof data === 'string' || data instanceof Blob) {
        this._files[path] = {
          mimeType: mimeType,
          data: data
        }
        resolve()
      } else {
        reject(new Error('MemoryFileSystem only supports utf-8 strings and blobs'))
      }
    })
  }

  readFile(path) {
    return new Promise((resolve, reject) => {
      let file = this._files[path]
      if (file) {
        resolve(file.data)
      } else {
        reject(new Error('File not found'))
      }
    })
  }

  /*
    Returns true if archive has unsaved changes
  */
  isDirty() {

  }

  /*
    IDEA: Back up the archive in order to recover unsaved changes later
  */
  backup() {

  }

  /*
    IDEA: Recover archive: Use the backed up version that has the unsaved changes
  */
  recover() {

  }

  /*
    IDEA: Returns true if there are recoverable (unsaved) files
  */
  canRecover() {

  }

  /*
    Save the archive to the remote. For the MemoryArchive there is no Remotes
    so we just simulate it.

    In real world archives remotes can be: Github, SFTP, the local file system, ...
  */
  save() {
    return new Promise((resolve) => {
      resolve()
    })
  }
}
