import { generateEmptySheetXML } from '../sheet/sheetHelpers'
import { EMPTY_ARTICLE_XML } from '../article/articleHelpers'

export function addNewDocument(archive, type) {
  let entries = archive.getDocumentEntries()
  let name
  let xml
  if (type === 'sheet') {
    let existingNames = new Set()
    entries.forEach(e => {
      if (e.type === 'sheet') {
        existingNames.add(e.name)
      }
    })
    name = `Sheet${existingNames.size+1}`
    xml = generateEmptySheetXML(100, 26)
  } else if (type === 'article') {
    let existingNames = new Set()
    entries.forEach(e => {
      if (e.type === 'article') {
        existingNames.add(e.name)
      }
    })
    name = `Article${existingNames.size+1}`
    xml = EMPTY_ARTICLE_XML
  }
  return archive.addDocument(type, name, xml)
}