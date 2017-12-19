import { Configurator, EditorSession } from 'substance'
import ArticleEditorPackage from './ArticleEditorPackage'

export default {
  load(html) {
    let configurator = new Configurator()
    configurator.import(ArticleEditorPackage)
    let importer = configurator.createImporter('html')
    let doc = importer.importDocument(html)
    let editorSession = new EditorSession(doc, { configurator })
    return editorSession
  }
}
