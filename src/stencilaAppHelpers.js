import { forEach } from 'substance'
import { JATSImportDialog } from 'substance-texture'
import Project from './project/Project'
import setupStencilaContext from './util/setupStencilaContext'
import SheetAdapter from './sheet/SheetAdapter'
import ArticleAdapter from './article/ArticleAdapter'
import { getSource, transformCellRangeExpressions } from './shared/cellHelpers'

export function _renderStencilaApp($$, app) {
  let el = $$('div').addClass('sc-app')
  let { archive, error } = app.state
  if (archive) {
    el.append(
      $$(Project, {
        documentArchive: archive
      })
    )
  } else if (error) {
    if (error.type === 'jats-import-error') {
      el.append(
        $$(JATSImportDialog, { errors: error.detail })
      )
    } else {
      el.append(
        'ERROR:',
        error.message
      )
    }
  } else {
    // LOADING...
  }
  return el
}

export function _setupStencilaChildContext(originalContext) {
  const context = setupStencilaContext()
  return Object.assign({}, originalContext, context)
}

export function _initStencilaContext(context) {
  return context.host.initialize()
}

export function _initStencilaArchive(archive, context) {
  // start the engine
  const ENGINE_REFRESH_INTERVAL = 10 // ms
  context.engine.run(ENGINE_REFRESH_INTERVAL)
  // when a document is renamed, transclusions must be updated
  _listenToDocumentRename(archive)
  // documents and sheets must be registered with the engine
  // and hooks for structural sheet updates must be established
  // to update transclusions.
  let entries = archive.getDocumentEntries()
  forEach(entries, entry => {
    _connectDocumentToEngine(archive, entry.id, context)
  })
  return Promise.resolve(archive)
}

// Connects documents with the Cell Engine
// and registers hooks to update transclusions.
export function _connectDocumentToEngine(archive, documentId, { engine }) {
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
      _listenForStructuralSheetUpdates(archive, editorSession)
      break
    }
    default:
      //
  }
  if (Adapter) {
    Adapter.connect(engine, editorSession, docId, name)
  }
}

function _listenToDocumentRename(archive) {
  let editorSession = archive.getEditorSession('manifest')
  editorSession.on('update', _onManifestChange.bind(null, archive), null, { resource: 'document' })
}

function _listenForStructuralSheetUpdates(archive, editorSession) {
  editorSession.on('update', _onSheetChange.bind(null, archive, editorSession.getDocument()), null, { resource: 'document' })
}

function _onManifestChange(archive, change) {
  let action = change.info.action
  switch(action) {
    case 'renameDocument': {
      console.log('TODO: transform transclusions')
      break
    }
    default:
      //
  }
}

function _onSheetChange(archive, doc, change) {
  let action = change.info.action
  switch(action) {
    case 'insertRows': {
      const { pos, count } = change.info
      _updateTransclusionsInArchive(archive, doc.id, { dim: 'row', pos, count })
      break
    }
    case 'deleteRows': {
      const { pos, count } = change.info
      _updateTransclusionsInArchive(archive, doc.id, { dim: 'row', pos, count: -count })
      break
    }
    case 'insertCols': {
      const { pos, count } = change.info
      _updateTransclusionsInArchive(archive, doc.id, { dim: 'col', pos, count })
      break
    }
    case 'deleteCols': {
      const { pos, count } = change.info
      _updateTransclusionsInArchive(archive, doc.id, { dim: 'col', pos, count: -count })
      break
    }
    default:
      //
  }
}

function _updateTransclusionsInArchive(archive, targetId, params) {
  let entries = archive.getDocumentEntries()
  entries.forEach(entry => {
    const id = entry.id
    if (id === targetId) return
    let documentType = archive.getDocumentType(id)
    switch (documentType) {
      case 'article':
      case 'sheet': {
        let editorSession = archive.getEditorSession(id)
        _updateTransclusionsInDocument(documentType, editorSession, targetId, params)
        break
      }
      default:
        //
    }
  })
}

function _updateTransclusionsInDocument(documentType, editorSession, targetId, params) {
  let doc = editorSession.getDocument()
  let cells = _getCellsWithTransclusions(doc, targetId)
  let updates = new Map()
  for (var i = 0; i < cells.length; i++) {
    const cell = cells[i]
    let source = getSource(cell)
    let newSource = transformCellRangeExpressions(source, params)
    if (newSource !== source) {
      updates.set(cell.id, newSource)
    }
  }
  if (updates.size > 0) {
    editorSession.transaction(tx => {
      updates.forEach((newSource, id) => {
        let node = tx.get(id)
        if (documentType === 'article') {
          node = node.find('source-code')
        }
        node.setText(newSource)
      })
    }, { history: false })
    // ATTENTION: we trying to manipulate the Undo/Redo-History
    // so the user does not corrupt cells via undo/redo
    // TODO: if we had finer, more accurate ops for updating cells
    // the implicit is similar to a change coming in as during real-time collab
    // and this could be solved using 'rebase'
    _eliminateOpsFromHistory(editorSession._history, updates)
  }
}

function _eliminateOpsFromHistory(history, ids) {
  _eliminateOpsFromChanges(history.doneChanges, ids)
  _eliminateOpsFromChanges(history.undoneChanges, ids)
}

function _eliminateOpsFromChanges(changes, ids) {
  for (let i = 0; i < changes.length; i++) {
    let change = changes[i]
    for (let j = change.ops.length - 1; j >= 0; j--) {
      let op = change.ops[j]
      if (ids.has(op.path[0])) {
        change.ops.splice(j, 1)
      }
    }
  }
}


function _getCellsWithTransclusions(doc, targetDocId) { // eslint-disable-line
  // TODO: for now we do filter cells
  // we would need some kind of indexing
  return doc.findAll('cell')
}
