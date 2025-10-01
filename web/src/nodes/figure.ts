import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `Figure`
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/figure.md
 */
@customElement('stencila-figure')
@withTwind()
export class Figure extends Entity {
  @property()
  label?: string;

  @property()
  labelAutomatically?: string;

  /**
   * Store original img styles for restoration after print
   */
  private originalImgStyles: Map<HTMLElement, {
    width: string;
    height: string;
    display: string;
    marginLeft: string;
    marginRight: string
  }> = new Map()

  private onBeforePrint = () => {
    // If necessary, scale images within the figure so that the figure,
    // including images and caption fits on a page

    // Maximum page height for print (in pixels)
    // Read from CSS variable --page-content-height, which accounts for page size,
    // margins, and padding. Falls back to 1000 if not defined.
    const pageContentHeight = getComputedStyle(document.documentElement)
      .getPropertyValue('--page-content-height')
      .trim()
    const maxFigureHeight = pageContentHeight
      ? parseFloat(pageContentHeight)
      : 1000

    // Query light DOM for slotted figure element
    const figure = this.querySelector('figure[slot="content"]') as HTMLElement
    if (!figure) {
      console.warn('stencila-figure: No figure element found in light DOM')
      return
    }

    // Create temporary style to simulate print media for accurate measurements
    // This ensures we measure caption heights with print font sizes, line heights, and widths
    const printStyle = document.createElement('style')
    printStyle.textContent = `
      :root {
        --content-spacing: var(--content-spacing-print);
        --text-font-size: var(--text-font-size-print);
        --text-line-height: var(--text-line-height-print);
        --figure-caption-spacing-top: var(--caption-spacing-print);
        --figure-caption-line-height: var(--caption-line-height-print);
      }
      stencila-figure,
      stencila-figure > figure,
      stencila-figure > figure > figcaption {
        width: var(--page-content-width);
      }
    `
    document.head.appendChild(printStyle)

    // Force layout calculation to ensure print styles are applied
    figure.getBoundingClientRect()

    // Get all image wrappers and images
    const imageObjects = Array.from(figure.querySelectorAll('stencila-image-object')) as HTMLElement[]
    const imgs = Array.from(figure.querySelectorAll('img')) as HTMLElement[]

    if (imgs.length === 0) {
      document.head.removeChild(printStyle)
      return
    }

    // Measure heights with print styles active
    // Use wrapper heights to get accurate space occupied including margins/padding
    const figureHeight = figure.offsetHeight
    const imagesHeight = imageObjects.length > 0
      ? imageObjects.reduce((sum, wrapper) => sum + wrapper.offsetHeight, 0)
      : imgs.reduce((sum, img) => sum + img.offsetHeight, 0)

    // Remove temporary print style
    document.head.removeChild(printStyle)

    /*
    const figureLabel = this.label || this.labelAutomatically || 'unlabeled'
    console.log(`Figure ${figureLabel} - onBeforePrint measurements:`, {
      figureHeight,
      maxFigureHeight,
      imageObjectCount: imageObjects.length,
      willScale: figureHeight > maxFigureHeight
    })
    */

    // If figure exceeds max height, scale the images
    if (figureHeight > maxFigureHeight) {
      // Calculate scale factor accounting for caption and spacing staying constant
      // newFigureHeight = (imagesHeight × scaleFactor) + captionHeight + extraSpacing
      // Solving for scaleFactor when newFigureHeight = maxFigureHeight:
      const scaleFactor = (maxFigureHeight - figureHeight + imagesHeight) / imagesHeight

      //console.log(`Figure ${figureLabel} - Scaling images:`, { scaleFactor })

      // Scale each image directly by setting width and height
      imgs.forEach((img) => {
        // Store original styles
        this.originalImgStyles.set(img, {
          width: img.style.width || '',
          height: img.style.height || '',
          display: img.style.display || '',
          marginLeft: img.style.marginLeft || '',
          marginRight: img.style.marginRight || ''
        })

        // Get current dimensions
        const currentWidth = img.offsetWidth
        const currentHeight = img.offsetHeight

        // Set scaled dimensions and center the image
        img.style.width = `${currentWidth * scaleFactor}px`
        img.style.height = `${currentHeight * scaleFactor}px`
        img.style.display = 'block'
        img.style.marginLeft = 'auto'
        img.style.marginRight = 'auto'
      })
    }
  }

  private onAfterPrint = () => {
    // Restore original styles for all img elements
    this.originalImgStyles.forEach((originalStyles, img) => {
      img.style.width = originalStyles.width
      img.style.height = originalStyles.height
      img.style.display = originalStyles.display
      img.style.marginLeft = originalStyles.marginLeft
      img.style.marginRight = originalStyles.marginRight
    })
    this.originalImgStyles.clear()
  }

  override connectedCallback() {
    super.connectedCallback()
    window.addEventListener('beforeprint', this.onBeforePrint)
    window.addEventListener('afterprint', this.onAfterPrint)
  }

  override disconnectedCallback() {
    window.removeEventListener('beforeprint', this.onBeforePrint)
    window.removeEventListener('afterprint', this.onAfterPrint)
    super.disconnectedCallback()
  }

  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    if (this.isWithinModelChatMessage()) {
      return this.renderCardWithChatAction()
    }

    return this.renderCard()
  }

  override renderCard() {
    return html`
      <stencila-ui-block-on-demand
        type="Figure"
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${this.hasRoot()}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Figure">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }

  private renderContent() {
    return html`
      <slot name="id"></slot>
      <slot name="content"></slot>
    `
  }
}
