/*
  In-memory buffer (cmp. mini filesytem) for representing Substance together
  with assets

  TODO: This needs to be rethought
*/
export default class MemoryBuffer {
  /*
    Takes a vfs with multiple publications, each in a folder.
    The publicationId is used as a scope
  */
  constructor(vfs, publicationId) {
    this.publicationId = publicationId
    this.vfs = vfs
  }

  /*
    File data must either be a utf8 string or a blob object
  */
  writeFile(/*path, mimeType, data*/) {
    throw new Error('Not yet implemented.')
  }

  readFile(path) {
    return new Promise((resolve, reject) => {
      let file = this.vfs.readFileSync(this.publicationId+"/"+path)
      if (file) {
        resolve(file)
      } else {
        reject(new Error('File not found'))
      }
    })
  }

}
