import SheetAdapter from '../sheet/SheetAdapter'
import ArticleAdapter from '../article/ArticleAdapter'

// Connects documents with the Cell Engine
// and registers hooks to update transclusions.
export default function _connectDocumentToEngine (engine, archive, documentId) {
  let manifest = archive.getEditorSession('manifest').getDocument()
  let docEntry = manifest.get(documentId)
  let editorSession = archive.getEditorSession(documentId)
  let docType = docEntry.attr('type')
  let name = docEntry.attr('name')
  let docId = docEntry.id
  let Adapter
  switch (docType) {
    case 'article': {
      Adapter = ArticleAdapter
      break
    }
    case 'sheet': {
      Adapter = SheetAdapter
      break
    }
    default:
      //
  }
  if (Adapter) {
    Adapter.connect(engine, editorSession, docId, name)
  }
}
