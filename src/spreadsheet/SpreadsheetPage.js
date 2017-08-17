import { Component, EditorSession, Configurator } from 'substance'
import SpreadsheetEditor from './SpreadsheetEditor'
import SpreadsheetPackage from './SpreadsheetPackage'

export default class SpreadsheetPage extends Component {

  getInitialState() {
    this._configurator = new Configurator().import(SpreadsheetPackage)

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
  }

  getChildContext() {
    return {
      editorSession: this.state.editorSession,
      configurator: this._configurator
    }
  }

  render($$) {
    const editorSession = this.getEditorSession()
    let el = $$('div').addClass('sc-spreadsheet-page')
    if (!editorSession) {
      el.text('Loading...')
    } else {
      el.append($$(SpreadsheetEditor, {
        editorSession
      }))
    }
    return el
  }

  getEditorSession() {
    return this.state.editorSession
  }

  getSheet() {
    return this.state.sheet
  }

  _loadBuffer() {
    // TODO: implement this
  }

  _initializeEditorSession(sheet) {
    let editorSession = new EditorSession(sheet, {
      configurator: this._configurator,
      context: {
        host: this.props.host
      }
    })
    this.extendState({
      editorSession,
      sheet
    })
  }
}