import { prettyPrintXML, PersistedDocumentArchive } from 'substance'
import { PubMetaLoader, JATSExporter } from 'substance-texture'
import SheetLoader from './sheet/SheetLoader'
import ArticleLoader from './article/ArticleLoader'

export default class StencilaArchive extends PersistedDocumentArchive {

  _loadDocument(type, record, sessions) {
    switch (type) {
      case 'application/jats4m': {
        // FIXME: we should not mix ingestion and regular loading
        // I.e. importing JATS4M should work without a pub-meta
        let pubMetaSession = PubMetaLoader.load()
        // HACK: we need to think about how to generalize this
        sessions['pub-meta'] = pubMetaSession
        // let dom = substance.DefaultDOMElement.parseXML(record.data)
        // console.log(prettyPrintXML(dom))
        // debugger
        return ArticleLoader.load(record.data, {
          pubMetaDb: pubMetaSession.getDocument(),
          archive: this
        })
      }
      case 'application/sheetml': {
        return SheetLoader.load(record.data)
      }
      default:
        throw new Error('Unsupported document type')
    }
  }

  _exportDocument(type, session, sessions) {
    switch (type) {
      case 'application/jats4m': {
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
        // let xmlStr = prettyPrintXML(res.dom)
        let xmlStr = res.dom.serialize()
        return xmlStr
      }
      case 'application/sheetml': {
        let dom = session.getDocument().toXML()
        let xmlStr = prettyPrintXML(dom)
        return xmlStr
      }
      default:
        throw new Error('Unsupported document type')
    }
  }

  getDocumentEntries() {
    let manifest = this.getEditorSession('manifest').getDocument()
    let documents = manifest.findAll('documents > document')
    return documents.map(doc => {
      return {
        id: doc.id,
        name: doc.attr('name'),
        type: doc.attr('type'),
        editorSession: this.getEditorSession(doc.id)
      }
    })
  }
}
