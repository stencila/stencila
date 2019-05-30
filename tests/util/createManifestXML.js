import { DefaultDOMElement as DOM } from 'substance'

const EMPTY_MANIFEST = `<?xml version="1.0" encoding="UTF-8"?>
<dar>
  <documents>
  </documents>
  <assets>
  </assets>
</dar>`

export default function createManifestXML(docSpecs, assetSpecs) {
  let doc = DOM.parseXML(EMPTY_MANIFEST)
  let $$ = doc.createElement.bind(doc)
  if (docSpecs) {
    let documents = doc.find('documents')
    docSpecs.forEach(spec => {
      documents.append(
        $$('document').attr({
          id: spec.id,
          name: spec.name,
          type: spec.type,
          path: spec.path
        })
      )
    })
  }
  if (assetSpecs) {
    let assets = doc.find('assets')
    assetSpecs.forEach(spec => {
      assets.append(
        $$('asset').attr({
          id: spec.id,
          type: spec.type,
          path: spec.path
        })
      )
    })
  }
  return doc.serialize()
}