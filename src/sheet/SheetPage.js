import { Component, EditorSession, Configurator } from 'substance'
import SheetEditor from './SheetEditor'
import SheetPackage from './SheetPackage'

export default class SheetPage extends Component {

  getInitialState() {
    this._configurator = new Configurator().import(SheetPackage)

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
    this.extendState({
      editorSession,
      sheet
    })
  }
}