import { EditorSession, Configurator } from 'substance'
import SheetPackage from './SheetPackage'
import SheetSchema from './SheetSchema'

export default {
  load(xml, context) {
    let configurator = new Configurator()
    configurator.import(SheetPackage)
    let importer = configurator.createImporter(SheetSchema.getName())

    let doc = importer.importDocument(xml)
    let editorSession = new EditorSession(doc, {
      configurator,
      context
    })
    return editorSession
  }
}
