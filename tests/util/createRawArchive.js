import createManifestXML from './createManifestXML'
import createArticleXML from './createArticleXML'
import createSheetXML from './createSheetXML'

/*
  Creates a raw archive from a simplified definition of documents
  used within the test-suite
*/
export default function createRawArchive(specs) {
  let rawArchive = {
    version: 0
  }
  let resources = {}
  let docSpecs = []
  specs.forEach(record => {
    let type = record.type
    let path = record.path
    switch(type) {
      case 'article': {
        resources[path] = {
          encoding: 'utf8',
          data: createArticleXML(record),
          type: 'application/xml'
        }
        docSpecs.push(record)
        break
      }
      case 'sheet': {
        resources[path] = {
          encoding: 'utf8',
          data: createSheetXML(record),
          type: 'application/xml'
        }
        docSpecs.push(record)
        break
      }
      default:
        console.error(`resource type '${type}' not supported yet`)
    }
  })
  resources['manifest.xml'] = {
    encoding: 'utf8',
    data: createManifestXML(docSpecs),
    type: 'application/xml'
  }
  rawArchive.resources = resources
  return rawArchive
}