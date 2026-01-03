import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

/**
 * Breadcrumbs navigation component
 *
 * A Light DOM component that displays a breadcrumb trail for navigation.
 * The HTML is server-rendered for SEO and accessibility.
 *
 * Features:
 * - Semantic HTML with proper ARIA attributes
 * - CSS-based separator styling
 * - Current page indication
 */
@customElement('stencila-breadcrumbs')
export class StencilaBreadcrumbs extends LitElement {
  /**
   * Use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-breadcrumbs': StencilaBreadcrumbs
  }
}
