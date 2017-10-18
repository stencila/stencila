import { platform, DefaultDOMElement, AbstractEditor, Toolbar } from 'substance'
import SheetLinter from './SheetLinter'
import SheetStatusBar from './SheetStatusBar'

export default class SheetEditor extends AbstractEditor {

  constructor(...args) {
    super(...args)

    this.__onResize = this.__onResize.bind(this)

    const sheet = this.getDocument()
    this.linter = new SheetLinter(sheet, this.getEditorSession())
  }

  getChildContext() {
    let editorSession = this.props.editorSession
    return Object.assign({}, super.getChildContext(), {
      issueManager: editorSession.issueManager
    })
  }

  getInitialState() {
    return {
      showConsole: false,
      consoleContent: null
    }
  }

  didMount() {
    // always render a second time to render for the real element dimensions
    this.rerender()
    super.didMount()
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).on('resize', this._onResize, this)
    }
    this.linter.start()
  }

  dispose() {
    super.dispose()
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).off(this)
    }
  }

  render($$) {
    let el = $$('div').addClass('sc-sheet-editor')
    el.append(
      this._renderToolbar($$),
      this._renderContent($$),
      this._renderStatusbar($$)
    )
    return el
  }

  _renderToolbar($$) {
    const configurator = this.getConfigurator()
    return $$(Toolbar, {
      toolPanel: configurator.getToolPanel('toolbar')
    }).ref('toolbar')
  }

  _renderContent($$) {
    let el = $$('div').addClass('se-body')
    el.append(
      this._renderSheet($$)
    )
    el.append(
      this._renderConsole($$)
    )
    return el
  }

  _renderSheet($$) {
    const sheet = this.getDocument()
    const linter = this.linter
    // only rendering the sheet when mounted
    // so that we have real width and height
    if (this.isMounted()) {
      const SheetComponent = this.getComponent('sheet')
      return $$(SheetComponent, {
        sheet, linter
      }).ref('sheet')
    } else {
      return $$('div')
    }
  }

  _renderConsole($$) {
    let el = $$('div').addClass('se-console')
    if (this.state.showConsole) {
      let ConsoleContent = this.getComponent(this.state.consoleContent)
      el.append(
        $$(ConsoleContent, { editor: this })
      )
    }
    return el
  }

  _renderStatusbar($$) {
    return $$(SheetStatusBar, {}).ref('sheet-statusbar')
  }

  getLinter() {
    return this.linter
  }

  getIssues() {
    let editorSession = this.props.editorSession
    let issueManager = editorSession.issueManager
    return issueManager.getIssues('linter')
  }

  getWidth() {
    if (this.el) {
      return this.el.getWidth()
    } else {
      return 1000
    }
  }

  getHeight() {
    if (this.el) {
      return this.el.getHeight()
    } else {
      return 750
    }
  }

  toggleConsole(consoleContent) {
    if (this.state.showConsole && this.state.consoleContent === consoleContent) {
      this.setState({
        showConsole: false
      })
    } else {
      this.setState({
        showConsole: true,
        consoleContent
      })
    }
  }

  _onResize() {
    if (platform.inBrowser) {
      if (!this._rafId) {
        this._rafId = window.requestAnimationFrame(this.__onResize)
      }
    }
  }

  __onResize() {
    this._rafId = null
    this.refs.sheet.resize()
  }

}
