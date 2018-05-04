import { DefaultDOMElement as DOM } from 'substance'
import { EMPTY_ARTICLE_XML } from '../../src/article/articleHelpers'

export default function createRawArticle(spec) {
  let doc = DOM.parseXML(EMPTY_ARTICLE_XML)
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