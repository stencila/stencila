import { platform, DefaultDOMElement, AbstractEditor, Toolbar } from 'substance'
import SheetComponent from './SheetComponent'
import SheetLinter from './SheetLinter'

export default class SheetEditor extends AbstractEditor {

  constructor(...args) {
    super(...args)

    this.__onResize = this.__onResize.bind(this)

    const sheet = this.getDocument()
    this.linter = new SheetLinter(sheet, this.getEditorSession())
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
      this._renderSheet($$),
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

  _renderSheet($$) {
    const sheet = this.getDocument()
    // only rendering the sheet when mounted
    // so that we have real width and height
    if (this.isMounted()) {
      return $$(SheetComponent, {
        sheet
      }).ref('sheet')
    } else {
      return $$('div')
    }
  }

  _renderStatusbar($$) {
    return $$('div').addClass('se-statusbar').text('STATUSBAR')
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

  getLinter() {
    return this.linter
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

}