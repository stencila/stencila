import { Component, EditorSession } from 'substance'
import {
  EditorPackage as TextureEditorPackage,
  createEntityDbSession,
  JATSExporter
} from 'substance-texture'
import Engine from '../engine/Engine'
import DocumentEngineAdapter from '../document/DocumentEngineAdapter'


export default class ArticlePage extends Component {

  constructor(parent, props) {
    super(parent, props)

    const host = props.host
    this.functionManager = host.functionManager
    this.engine = new Engine(host)

    // TODO: we need to see where the best place is for the entityDb
    this.entityDbSession = createEntityDbSession()
    this.entityDb = this.entityDbSession.getDocument()
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
      configurator: this.props.configurator,
      functionManager: this.functionManager,
      cellEngine: this.engine,
      host: this.props.host,
      exporter: {
        export(dom) {
          const entityDb = this.entityDb
          let jatsExporter = new JATSExporter()
          return jatsExporter.export(dom, { entityDb })
        }
      },
      entityDb: this.entityDb,
      entityDbSession: this.entityDbSession,
      // TODO: Update components to use entityDb, entityDbSession instead.
      get db() {
        console.warn('DEPRECATED: use context.entityDb instead')
        return this.entityDb
      },
      get dbSession() {
        console.warn('DEPRECATED: use context.entityDbSession instead')
        return this.entityDbSession
      }
    }
  }

  render($$) {
    const editorSession = this.getEditorSession()
    let el = $$('div').addClass('sc-article-page')
    if (!editorSession) {
      el.text('Loading...')
    } else {
      el.append(
        $$(TextureEditorPackage.Editor, {editorSession})
          .addClass('sc-document-editor')
      )
    }
    return el
  }

  getEditorSession() {
    return this.state.editorSession
  }

  _initializeEditorSession(article) {
    let editorSession = new EditorSession(article, {
      configurator: this.props.configurator,
      context: this.getChildContext()
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
