import { prettyPrintXML } from 'substance'
import { JATSExporter } from 'substance-texture'
import persistCellStates from './persistCellStates'

export default {
  export (session, { sessions }) {
    // FIXME: hard-coded, and thus bad
    // TODO: export only those resources which have been changed
    // Also we need to
    let jatsExporter = new JATSExporter()
    let pubMetaDb = sessions['pub-meta'].getDocument()
    let doc = session.getDocument()
    let dom = doc.toXML()

    let res = jatsExporter.export(dom, { pubMetaDb, doc })
    persistCellStates(doc, res.dom)

    console.info('saving jats', res.dom.getNativeElement())
    // TODO: bring back pretty printing (currently messes up CDATA content)
    let xmlStr = prettyPrintXML(res.dom)
    return xmlStr
  }
}
