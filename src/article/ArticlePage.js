import { Component, EditorSession } from 'substance'
import { EditorPackage as TextureEditorPackage } from 'substance-texture'

import Engine from '../engine/NewEngine'
import DocumentEngineAdapter from '../document/DocumentEngineAdapter'

export default class ArticlePage extends Component {

  constructor(parent, props) {
    super(parent, props)

    const host = props.host
    this.functionManager = host.functionManager
    this.engine = new Engine(host)
  }

  getInitialState() {
    return {
      editorSession: null,
      article: null
    }
  }

  didMount() {
    this._initializeEditorSession(this.props.article)
  }

  getChildContext() {
    return {
      app: this,
      editorSession: this.state.editorSession,
      configurator: this.props.configurator
    }
  }

  render($$) {
    const editorSession = this.getEditorSession()
    let el = $$('div').addClass('sc-article-page')
    if (!editorSession) {
      el.text('Loading...')
    } else {
      el.append(
        $$(TextureEditorPackage.Editor, {editorSession}
      ).addClass('sc-document-editor'))
    }
    return el
  }

  getEditorSession() {
    return this.state.editorSession
  }

  _initializeEditorSession(article) {
    let editorSession = new EditorSession(article, {
      configurator: this.props.configurator,
      context: {
        app: this,
        host: this.props.host
      }
    })

    let engineAdapter = new DocumentEngineAdapter(editorSession)
    engineAdapter.connect(this.engine)
    this.engine.editorSession = editorSession

    this.extendState({
      editorSession,
      article
    })
  }
}

