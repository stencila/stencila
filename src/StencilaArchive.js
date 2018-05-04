import { prettyPrintXML } from 'substance'
import { JATSExporter, TextureArchive } from 'substance-texture'
import ArticleLoader from './article/ArticleLoader'
import SheetLoader from './sheet/SheetLoader'

export default class StencilaArchive extends TextureArchive {

  constructor(storage, buffer, context) {
    super(storage, buffer)
    this._context = context
  }

  load(archiveId) {
    return super.load(archiveId)
      .then(() => {
        this._fixNameCollisions()
        return this
      })
  }

  _loadDocument(type, record, sessions) {
    let context = this._context
    let editorSession
    switch (type) {
      case 'article': {
        context = Object.assign({}, this._context, {
          pubMetaDb: sessions['pub-meta'].getDocument(),
          archive: this
        })
        editorSession = ArticleLoader.load(record.data, context)
        break
      }
      case 'sheet': {
        editorSession = SheetLoader.load(record.data, context)
        break
      }
      default:
        throw new Error('Unsupported document type')
    }
    let doc = editorSession.getDocument()
    doc.documentType = type
    return editorSession
  }

  _fixNameCollisions() {
    let manifestSession = this._sessions['manifest']
    let entries = manifestSession.getDocument().getDocumentEntries()
    // TODO: this should also be done in DAR in general
    let names = new Set()
    entries.forEach(entry => {
      let name = entry.name
      // fixup the name as long there are collisions
      while (name && names.has(name)) {
        name = name + '(duplicate)'
      }
      if (entry.name !== name) {
        manifestSession.transaction(tx => {
          let docEntry = tx.get(entry.id)
          docEntry.attr({name})
        }, { action: 'renameDocument' })
      }
      names.add(entry.name)
    })
  }

  _exportDocument(type, session, sessions) {
    switch (type) {
      case 'article': {
        // FIXME: hard-coded, and thus bad
        // TODO: export only those resources which have been changed
        // Also we need to
        let jatsExporter = new JATSExporter()
        let pubMetaDb = sessions['pub-meta'].getDocument()
        let doc = session.getDocument()
        let dom = doc.toXML()
        let res = jatsExporter.export(dom, { pubMetaDb, doc })
        console.info('saving jats', res.dom.getNativeElement())
        // TODO: bring back pretty printing (currently messes up CDATA content)
        let xmlStr = prettyPrintXML(res.dom)
        //let xmlStr = res.dom.serialize()
        return xmlStr
      }
      case 'sheet': {
        let dom = session.getDocument().toXML()
        let xmlStr = prettyPrintXML(dom)
        return xmlStr
      }
      default:
        throw new Error('Unsupported document type')
    }
  }

  /*
    We use the name of the first document
  */
  getTitle() {
    let entries = this.getDocumentEntries()
    let firstEntry = entries[0]
    return firstEntry.name || firstEntry.id
  }

  getDocumentType(documentId) {
    let editorSession = this.getEditorSession(documentId)
    let doc = editorSession.getDocument()
    return doc.documentType
  }

  // added `info.action = 'addDocument'`
  // TODO: this should go into substance.PersistedDocumentArchive
  _addDocumentRecord(documentId, type, name, path) {
    this._sessions.manifest.transaction(tx => {
      let documents = tx.find('documents')
      let docEntry = tx.createElement('document', { id: documentId }).attr({
        name: name,
        path: path,
        type: type
      })
      documents.appendChild(docEntry)
    }, { action: 'addDocument' })
  }

  // added `info.action = 'renameDocument'`
  // TODO: this should go into substance.PersistedDocumentArchive
  renameDocument(documentId, name) {
    this._sessions.manifest.transaction(tx => {
      let docEntry = tx.find(`#${documentId}`)
      docEntry.attr({name})
    }, { action: 'renameDocument' })
  }
}
