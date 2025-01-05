import { css, html, LitElement } from 'lit'
import { customElement, state } from 'lit/decorators'
import { unsafeSVG } from 'lit/directives/unsafe-svg'

import left from './logo-circle-left.svg'
import middle from './logo-circle-middle.svg'
import right from './logo-circle-right.svg'

@customElement('stencila-animated-logo')
export class StencilaAnimatedLogo extends LitElement {
  /**
   * Array of svg strings to be cycled
   */
  private readonly svgs = [left, middle, right]

  /**
   * current interval instance
   */
  private interval: NodeJS.Timeout = null

  /**
   * Index of the icon current displaying
   */
  @state()
  currentIndex: number = 0

  start() {
    this.interval = setInterval(() => {
      this.currentIndex = (this.currentIndex + 1) % this.svgs.length
    }, 700)
  }

  stop() {
    if (this.interval) {
      clearInterval(this.interval)
      this.interval = null
    }
  }

  static override styles = css`
    :host {
      display: block;
      width: 1em;
      height: 1em;
      box-sizing: content-box !important;
    }

    svg {
      display: block;
      height: 100%;
      width: 100%;
    }
  `

  override connectedCallback(): void {
    super.connectedCallback()
    this.start()
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    this.stop()
  }

  protected override render() {
    return html`${unsafeSVG(this.svgs[this.currentIndex])}`
  }
}
