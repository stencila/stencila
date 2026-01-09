import { LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

type CopyState = 'idle' | 'loading' | 'success' | 'error'

/**
 * Copy markdown component
 *
 * Fetches the page's markdown file and copies it to the clipboard.
 * Provides visual feedback during loading, success, and error states.
 */
@customElement('stencila-copy-markdown')
export class StencilaCopyMarkdown extends LitElement {
  /**
   * URL of the markdown file to fetch
   */
  @property({ type: String, attribute: 'data-url' })
  url = ''

  /**
   * Current state of the copy operation
   */
  @state()
  private copyState: CopyState = 'idle'

  /**
   * Original button HTML for restoration
   */
  private originalButtonHtml = ''

  /**
   * Timeout for resetting state after feedback
   */
  private resetTimeout: ReturnType<typeof setTimeout> | null = null

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Store original button content
    const button = this.querySelector('button')
    if (button) {
      this.originalButtonHtml = button.innerHTML
      button.addEventListener('click', this.handleClick)
      button.addEventListener('keydown', this.handleKeydown)
    }
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    if (this.resetTimeout) {
      clearTimeout(this.resetTimeout)
    }

    const button = this.querySelector('button')
    if (button) {
      button.removeEventListener('click', this.handleClick)
      button.removeEventListener('keydown', this.handleKeydown)
    }
  }

  /**
   * Handle button click - fetch markdown and copy to clipboard
   */
  private handleClick = async () => {
    if (this.copyState === 'loading') {
      return // Prevent double-clicks
    }

    this.setCopyState('loading')

    try {
      // Fetch the markdown file
      const response = await fetch(this.url)
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }

      const markdown = await response.text()

      // Copy to clipboard
      await navigator.clipboard.writeText(markdown)

      this.setCopyState('success')
      this.showFeedback()
    } catch (error) {
      console.error('Failed to copy markdown:', error)
      this.setCopyState('error')
      this.showFeedback()
    }
  }

  /**
   * Handle keyboard activation
   */
  private handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      this.handleClick()
    }
  }

  /**
   * Set copy state and update button appearance
   */
  private setCopyState(state: CopyState) {
    this.copyState = state
    this.updateButtonAppearance()
  }

  /**
   * Update button appearance based on current state
   */
  private updateButtonAppearance() {
    const button = this.querySelector('button')
    if (!button) return

    button.setAttribute('data-state', this.copyState)

    switch (this.copyState) {
      case 'loading':
        button.innerHTML = '<span class="icon i-lucide:loader-2 animate-spin"></span>'
        button.setAttribute('aria-label', 'Copying...')
        break
      case 'success':
        button.innerHTML =
          '<span class="icon i-lucide:check"></span><span class="text">Copied!</span>'
        button.setAttribute('aria-label', 'Copied!')
        break
      case 'error':
        button.innerHTML =
          '<span class="icon i-lucide:x"></span><span class="text">Failed to copy</span>'
        button.setAttribute('aria-label', 'Failed to copy')
        break
      case 'idle':
      default:
        button.innerHTML = this.originalButtonHtml
        button.setAttribute('aria-label', 'Copy page as Markdown')
        break
    }
  }

  /**
   * Show feedback briefly then reset to idle
   */
  private showFeedback() {
    if (this.resetTimeout) {
      clearTimeout(this.resetTimeout)
    }

    this.resetTimeout = setTimeout(() => {
      this.setCopyState('idle')
    }, 2000)
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-copy-markdown': StencilaCopyMarkdown
  }
}
