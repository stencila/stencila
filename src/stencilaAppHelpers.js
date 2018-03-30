import { forEach } from 'substance'
import { JATSImportDialog } from 'substance-texture'

import Project from './project/Project'
import setupStencilaContext from './util/setupStencilaContext'

import SheetAdapter from './sheet/SheetAdapter'
import ArticleAdapter from './article/ArticleAdapter'

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

export function _initStencilaArchive(archive, {engine}) {
  // register documents and sheets with the engine
  let entries = archive.getDocumentEntries()
  forEach(entries, entry => {
    let { id, type } = entry
    let editorSession = archive.getEditorSession(id)
    let Adapter
    if (type === 'article') {
      Adapter = ArticleAdapter
    } else if (type === 'sheet') {
      Adapter = SheetAdapter
    }
    if (Adapter) {
      Adapter.connect(engine, editorSession, id)
    }
  })
  // start the engine
  const ENGINE_REFRESH_INTERVAL = 10 // ms
  engine.run(ENGINE_REFRESH_INTERVAL)
  return Promise.resolve(archive)
}
