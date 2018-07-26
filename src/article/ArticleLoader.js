import { EditorSession } from 'substance'
import { TextureConfigurator, JATSImporter } from 'substance-texture'
import ArticleEditorPackage from './ArticleEditorPackage'
import loadPersistedCellStates from './loadPersistedCellStates'

export default {
  load(xml, context) {
    let configurator = new TextureConfigurator()
    // TODO: it would make more sense to use a more generic configuration here (TextureJATSPackage)
    // But ATM EditorSession is owning all the managers. So we have to use the EditorPackage.
    configurator.import(ArticleEditorPackage)
    let jatsImporter = new JATSImporter()
    let jats = jatsImporter.import(xml, context)

    if (jats.hasErrored) {
      let err = new Error()
      err.type = 'jats-import-error'
      err.detail = jats.errors
      throw err
    }

    let importer = configurator.createImporter('texture-article')
    let doc = importer.importDocument(jats.dom)
    let editorSession = new EditorSession(doc, { configurator, context })
    // EXPERIMENTAL: taking persisted cell outputs to initialize cell state
    loadPersistedCellStates(doc)

    return editorSession
  }
}
