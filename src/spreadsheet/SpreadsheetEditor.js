import { platform, DefaultDOMElement, AbstractEditor, Toolbar } from 'substance'
import SpreadsheetComponent from './SpreadsheetComponent'

export default class SpreadsheetEditor extends AbstractEditor {

  constructor(...args) {
    super(...args)

    this.__onResize = this.__onResize.bind(this)
  }

  didMount() {
    // always render a second time to render for the real element dimensions
    this.rerender()

    super.didMount()
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).on('resize', this._onResize, this)
    }
  }

  dispose() {
    super.dispose()
    if (platform.inBrowser) {
      DefaultDOMElement.wrap(window).off(this)
    }
  }


  render($$) {
    let el = $$('div').addClass('sc-spreadsheet-editor')
    el.append(
      this._renderToolbar($$),
      this._renderSpreadsheet($$),
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

  _renderSpreadsheet($$) {
    const sheet = this.getDocument()
    // only rendering the spreadsheet when mounted
    // so that we have real width and height
    if (this.isMounted()) {
      return $$(SpreadsheetComponent, {
        sheet
      }).ref('spreadsheet')
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
    this.rerender()
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