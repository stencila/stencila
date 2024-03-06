import { html, LitElement } from 'lit'

import { DirectoryAction, directoryActionEvent } from '../clients/directory'
import { nodePatchEvent, NodePatch } from '../clients/nodes'
import { DocumentAccess, DocumentView } from '../types'

/**
 * Abstract base class for web components representing Stencila Schema `Entity` node types
 *
 * Given that the `Entity` node type is the ancestor of all other node types in the Stencila
 * Schema (other than "primitive" types), this is the ancestor class of all web components
 * for node types. As such it implements and few core methods commonly used.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md
 *
 * Note that this does not need to have an `id` property (as in the `Entity` schema)
 * because `id` is already a property of `HTMLElement` from which this is derived.
 */
export abstract class Entity extends LitElement {
  /**
   * Select the closest element matching a selector
   *
   * This is similar to the `closest` method of HTML elements but traverses
   * up out of the shadow root is necessary.
   *
   * Based on https://stackoverflow.com/questions/54520554/custom-element-getrootnode-closest-function-crossing-multiple-parent-shadowd
   */
  protected closestGlobally(selector: string): HTMLElement | null {
    function closest(
      elem: HTMLElement | Document | Window
    ): HTMLElement | null {
      if (!elem || elem === document || elem === window) return null
      const found = (elem as HTMLElement).closest(selector)
      // @ts-expect-error because `Node` has no host property
      return found ? found : closest(elem.getRootNode().host)
    }
    return closest(this)
  }

  /**
   * Get the name of the view that this web component is contained within
   *
   * This may be used by derived elements to alter their rendering and/or
   * behavior based on the view.
   */
  protected documentView(): DocumentView {
    return this.closestGlobally('[view]')?.getAttribute('view') as DocumentView
  }

  /**
   * Get the document access level of the view that this web component
   * is contained within
   *
   * This may be used by derived elements to alter their rendering and/or
   * behavior based on the view.
   */
  protected documentAccess(): DocumentAccess {
    return this.closestGlobally('[view]')?.getAttribute(
      'access'
    ) as DocumentAccess
  }

  /**
   * Patch the node that this web component represents
   *
   * Emits a `CustomEvent` containing a `NodePatch` which is forwarded by
   * the `NodesClient` to the document on the server.
   */
  protected patchNode(patch: NodePatch) {
    this.dispatchEvent(nodePatchEvent(patch))
  }

  /**
   * Convenience method to emit a directory action event
   */
  protected directoryAction(
    type: DirectoryAction['type'],
    path: string,
    to?: string
  ) {
    this.dispatchEvent(directoryActionEvent({ type, path, to }))
  }

  /**
   * Default rendering method that dispatches to a view specific method
   *
   * In general, derived components should not need to override this but
   * should instead override `render<VIEW>` methods.
   */
  override render() {
    const view = this.documentView()
    switch (view) {
      case 'static':
      case 'live':
        return this.renderStaticView()
      case 'interactive':
        return this.renderInteractiveView()
      case 'dynamic':
        return this.renderDynamicView()
      case 'visual':
        return this.renderVisualView()
      case 'source':
        return this.renderSourceView()
      default:
        return html`Not implemented`
    }
  }

  renderStaticView() {
    return html`Not implemented`
  }

  renderInteractiveView() {
    // Default is to render the static view since most
    // nodes will be the same in both views.
    return this.renderStaticView()
  }

  renderDynamicView() {
    return html`Not implemented`
  }

  renderVisualView() {
    return html`Not implemented`
  }

  renderSourceView() {
    return html`Not implemented`
  }
}
