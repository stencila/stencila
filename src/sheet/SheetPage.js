import { Component, EditorSession, Configurator } from 'substance'
import SheetEditor from './SheetEditor'
import SheetPackage from './SheetPackage'
import Engine from '../engine/Engine'
import SheetEngineAdapter from './SheetEngineAdapter'

export default class SheetPage extends Component {

  constructor(parent, props) {
    super(parent, props)

    const host = props.host
    this._configurator = new Configurator().import(SheetPackage)
    this.engine = new Engine(host)
    this.functionManager = host.functionManager
  }

  getInitialState() {
    return {
      editorSession: null,
      sheet: null
    }
  }

  didMount() {
    if (this.props.sheet) {
      this._initializeEditorSession(this.props.sheet)
    } else if (this.props.sheetId) {
      this._loadBuffer()
    }
  }

  didUpdate(oldProps) {
    if (this.props.sheet && oldProps.sheet !== this.props.sheet) {
      this.el.empty()
      this._initializeEditorSession(this.props.sheet)
    } else if (this.props.sheetId && oldProps.sheetId !== this.props.sheetId) {
      this.el.empty()
      this._loadBuffer()
    }
  }

  dispose() {
    // nothing
  }

  getChildContext() {
    return {
      app: this,
      editorSession: this.state.editorSession,
      configurator: this._configurator
    }
  }

  render($$) {
    const editorSession = this.getEditorSession()
    let el = $$('div').addClass('sc-sheet-page')
    if (!editorSession) {
      el.text('Loading...')
    } else {
      el.append($$(SheetEditor, { editorSession }).ref('editor'))
    }
    return el
  }

  getEditorSession() {
    return this.state.editorSession
  }

  getSheet() {
    return this.state.sheet
  }

  getSheetEditor() {
    return this.refs.editor
  }

  _loadBuffer() {
    // TODO: implement this
  }

  _initializeEditorSession(sheet) {
    let editorSession = new EditorSession(sheet, {
      configurator: this._configurator,
      context: {
        app: this,
        host: this.props.host
      }
    })

    editorSession.issueManager = editorSession.getManager('issue-manager')

    let engineAdapter = new SheetEngineAdapter(editorSession)
    engineAdapter.connect(this.engine)
    this.engine.editorSession = editorSession

    this.extendState({
      editorSession,
      sheet
    })
  }
}
