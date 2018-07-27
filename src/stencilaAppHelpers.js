import { JATSImportDialog } from 'substance-texture'
import Project from './project/Project'
import setupStencilaContext from './util/setupStencilaContext'

export function _renderStencilaApp ($$, app) {
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

export function _setupStencilaChildContext (originalContext) {
  const context = setupStencilaContext()
  return Object.assign({}, originalContext, context)
}

export function _initStencilaContext (context) {
  return context.host.initialize()
}

export { default as _initStencilaArchive } from './shared/_initStencilaArchive'
export { default as _connectDocumentToEngine } from './shared/_connectDocumentToEngine'
