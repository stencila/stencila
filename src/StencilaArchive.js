import { prettyPrintXML } from 'substance'
import { JATSExporter, TextureArchive } from 'substance-texture'
import ArticleLoader from './article/ArticleLoader'
import SheetLoader from './sheet/SheetLoader'

export default class StencilaArchive extends TextureArchive {

  _loadDocument(type, record, sessions) {
    switch (type) {
      case 'article': {
        return ArticleLoader.load(record.data, {
          pubMetaDb: sessions['pub-meta'].getDocument(),
          archive: this
        })
      }
      case 'sheet': {
        return SheetLoader.load(record.data)
      }
      default:
        throw new Error('Unsupported document type')
    }
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
}
