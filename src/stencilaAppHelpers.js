import { forEach } from 'substance'
import { JATSImportDialog } from 'substance-texture'
import Project from './project/Project'
import setupStencilaContext from './util/setupStencilaContext'
import SheetAdapter from './sheet/SheetAdapter'
import ArticleAdapter from './article/ArticleAdapter'
import { getSource, transformCellRangeExpressions, renameTransclusions } from './shared/cellHelpers'

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
  const engine = context.engine
  const ENGINE_REFRESH_INTERVAL = 10 // ms
  engine.run(ENGINE_REFRESH_INTERVAL)
  // when a document is renamed, transclusions must be updated
  _listenForDocumentRecordUpdates(archive, engine)
  // documents and sheets must be registered with the engine
  // and hooks for structural sheet updates must be established
  // to update transclusions.
  let entries = archive.getDocumentEntries()
  forEach(entries, entry => {
    _connectDocumentToEngine(engine, archive, entry.id)
  })
  return Promise.resolve(archive)
}

// Connects documents with the Cell Engine
// and registers hooks to update transclusions.
export function _connectDocumentToEngine(engine, archive, documentId) {
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

function _listenForDocumentRecordUpdates(archive, engine) {
  let editorSession = archive.getEditorSession('manifest')
  editorSession.on('update', _onManifestChange.bind(null, archive, engine), null, { resource: 'document' })
}

function _listenForStructuralSheetUpdates(archive, editorSession) {
  editorSession.on('update', _onSheetChange.bind(null, archive, editorSession.getDocument()), null, { resource: 'document' })
}

function _onManifestChange(archive, engine, change) {
  let action = change.info.action
  switch(action) {
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
        engine._setResourceName(docId, newName)
        _updateTransclusionsInArchive(archive, docId, action, { oldName, newName })
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

function _onSheetChange(archive, doc, change) {
  let action = change.info.action
  switch(action) {
    case 'insertRows': {
      const { pos, count } = change.info
      _updateTransclusionsInArchive(archive, doc.id, action, { dim: 'row', pos, count })
      break
    }
    case 'deleteRows': {
      const { pos, count } = change.info
      _updateTransclusionsInArchive(archive, doc.id, action, { dim: 'row', pos, count: -count })
      break
    }
    case 'insertCols': {
      const { pos, count } = change.info
      _updateTransclusionsInArchive(archive, doc.id, action, { dim: 'col', pos, count })
      break
    }
    case 'deleteCols': {
      const { pos, count } = change.info
      _updateTransclusionsInArchive(archive, doc.id, action, { dim: 'col', pos, count: -count })
      break
    }
    default:
      //
  }
}

function _updateTransclusionsInArchive(archive, targetId, action, params) {
  let entries = archive.getDocumentEntries()
  entries.forEach(entry => {
    const id = entry.id
    if (id === targetId) return
    let documentType = archive.getDocumentType(id)
    switch (documentType) {
      case 'article':
      case 'sheet': {
        let editorSession = archive.getEditorSession(id)
        _updateTransclusionsInDocument(documentType, editorSession, targetId, action, params)
        break
      }
      default:
        //
    }
  })
}

function _updateTransclusionsInDocument(documentType, editorSession, targetId, action, params) {
  let doc = editorSession.getDocument()
  let cells = _getCellsWithTransclusions(doc, targetId)
  let updates = new Map()
  for (var i = 0; i < cells.length; i++) {
    const cell = cells[i]
    let source = getSource(cell)
    let newSource
    if (action === 'renameDocument') {
      newSource = renameTransclusions(source, params.oldName, params.newName)
    } else {
      newSource = transformCellRangeExpressions(source, params)
    }
    if (newSource !== source) {
      updates.set(cell.id, newSource)
    }
  }
  if (updates.size > 0) {
    // TODO: it is a bit cumbersome, that article cells have a different layout
    // We should think about using a simplified model internally, and generate
    let affected = new Set()
    editorSession.transaction(tx => {
      updates.forEach((newSource, id) => {
        let node = tx.get(id)
        if (documentType === 'article') {
          node = node.find('source-code')
        }
        affected.add(node.id)
        node.setText(newSource)
      })
    }, { history: false })
    // ATTENTION: removing all changes from undo/redo history
    // that would conflict with the automatic update
    // TODO: instead of removing we could try to 'rebase' such changes
    _eliminateOpsFromHistory(editorSession._history, affected)
  }
}

function _eliminateOpsFromHistory(history, ids) {
  _eliminateOpsFromChanges(history.doneChanges, ids)
  _eliminateOpsFromChanges(history.undoneChanges, ids)
}

function _eliminateOpsFromChanges(changes, ids) {
  for (let i = changes.length-1; i >= 0; i--) {
    let change = changes[i]
    // remove all ops that change the cell
    for (let j = change.ops.length-1; j >= 0; j--) {
      let op = change.ops[j]
      if (ids.has(op.path[0])) {
        change.ops.splice(j, 1)
      }
    }
    // remove a change if it is NOP now
    if (change.ops.length === 0) {
      changes.splice(i, 1)
    }
  }
}

function _getCellsWithTransclusions(doc, targetDocId) { // eslint-disable-line
  // TODO: for now we do filter cells
  // we would need some kind of indexing
  return doc.findAll('cell')
}
