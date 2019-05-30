import { JATSImportDialog } from 'substance-texture'
import { Component } from 'substance'
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
    el.append(
      $$(Loading, {
        message: 'Providing runtime environment. This may take up to a few minutes.'
      })
    )
  }
  return el
}

class Loading extends Component {
  render($$) {
    let el = $$('div').addClass('sc-loading')
    el.append(
      $$('div').addClass('se-spinner').append(
        $$('div'),
        $$('div'),
        $$('div'),
        $$('div'),
        $$('div'),
        $$('div')
      ),
      $$('div').addClass('se-message').append(
        this.props.message
      )
    )
    return el
  }
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
