import { forEach } from 'substance'
import _connectDocumentToEngine from './_connectDocumentToEngine'

export default function _initStencilaArchive (archive, context) {
  const engine = context.host && context.host.engine
  if (engine) {
    // when a document is renamed, transclusions must be updated
    _listenForDocumentRecordUpdates(archive, engine)
    // documents and sheets must be registered with the engine
    // and hooks for structural sheet updates must be established
    // to update transclusions.
    let entries = archive.getDocumentEntries()
    forEach(entries, entry => {
      _connectDocumentToEngine(engine, archive, entry.id)
    })
  }
  return Promise.resolve(archive)
}

function _listenForDocumentRecordUpdates (archive, engine) {
  let editorSession = archive.getEditorSession('manifest')
  editorSession.on('update', _onManifestChange.bind(null, archive, engine), null, { resource: 'document' })
}

function _onManifestChange (archive, engine, change) {
  let action = change.info.action
  switch (action) {
    case 'renameDocument': {
      // extracting document id, old name and the new name
      // TODO: maybe we can create an API to access such documentChange informations
      let op = change.ops[0]
      let docId = op.path[0]
      let oldName = op.original
      let newName = op.val
      if (oldName !== newName) {
        // TODO: it would be nice, if this could be done by the respective
        // document/sheet adapter. However, ATM renaming is done on manifest only,
        // so there is no document level notion of the name.
        let resource = engine.getResource(docId)
        resource.rename(newName)
      }
      break
    }
    case 'addDocument': {
      let op = change.ops[0]
      let docId = op.path[0]
      _connectDocumentToEngine(engine, archive, docId)
      break
    }
    default:
      //
  }
}
