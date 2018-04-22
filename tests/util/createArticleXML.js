import { DefaultDOMElement as DOM } from 'substance'

// TODO: Texture should provide a low-level entry API like here
// to allow easier creation of article XML

let EMPTY_ARTICLE = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE article PUBLIC "-//NLM//DTD JATS (Z39.96) Journal Archiving and Interchange DTD v1.1d3 20150301//EN"  "JATS-archivearticle1.dtd">
<article xmlns:xlink="http://www.w3.org/1999/xlink">
  <front>
    <article-meta>
      <title-group>
        <article-title></article-title>
      </title-group>
      <abstract>
      </abstract>
    </article-meta>
  </front>
  <body>
  </body>
  <back>
  </back>
</article>`

export default function createRawArticle(spec) {
  let doc = DOM.parseXML(EMPTY_ARTICLE)
  let $$ = doc.createElement.bind(doc)
  let bodySpec = spec.body || []
  let body = doc.find('body')
  bodySpec.forEach(block=> {
    let el = DOM.parseSnippet(block, 'xml')
    // NOTE: because cells are so inconvient in JATS we use a simplified model here
    switch (el.tagName) {
      case 'cell': {
        let cell = el
        let lang = cell.attr('language')
        el = $$('code').attr({
          'specific-use': 'cell',
          'id': cell.attr('id')
        }).append(
          $$('named-content').append(
            $$('alternatives').append(
              $$('code').attr({
                'specific-use': 'source',
                'language': lang
              }).append(cell.getInnerHTML()),
              $$('code').attr({
                'specific-use': 'output',
              })
            )
          )
        )
        break
      }
      default:
        //
    }
    body.append(el)
  })
  return doc.serialize()
}