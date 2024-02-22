import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { DOMParser, Schema } from 'prosemirror-model'
import { EditorState } from 'prosemirror-state'
import {
  NodeViewConstructor,
  EditorView as ProseMirrorView,
} from 'prosemirror-view'

import { DomClient } from '../clients/dom'
import { ProseMirrorClient } from '../clients/prosemirror'
import '../nodes'
import type { DocumentId, DocumentAccess } from '../types'

import 'prosemirror-menu/style/menu.css'

import { ThemedView } from './themed'
import * as schemas from './visual/schemas'

/**
 * Visual editor for a document
 *
 * A view which, in addition to providing live updates of a document,
 * allows for the user to modify the prose and other node types in it
 * using a WYSIWYG editor.
 */
@customElement('stencila-visual-view')
export class VisualView extends ThemedView {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId

  /**
   * The access level of the view
   *
   * This property is passed through to the `NodesClient` but may also
   * be inspected by descendent WebComponents to determine their behavior.
   *
   * This will normally be one of `comment`, `suggest`, `edit`, `write`,
   * or `admin`.
   */
  @property()
  access: DocumentAccess = 'admin'

  /**
   * A read-only client which updates the document's DOM when the
   * document changes on the server
   */
  // @ts-expect-error "dom client is set, not read"
  private domClient: DomClient

  /**
   * A write-only client that transforms ProseMirror transactions to
   * node patches and sends them to the document on the server
   */
  private proseMirrorClient: ProseMirrorClient

  /**
   * A ProseMirror editor view which the client interacts with
   */
  // @ts-expect-error "prose mirror view is set, not read"
  private proseMirrorView: ProseMirrorView

  /**
   * Override so that clients are instantiated _after_ this
   * element has a document `[root]` element in its `renderRoot`.
   *
   * If the `doc` property has changed, destroy and create the view again
   */
  override update(changedProperties: Map<string, string | boolean>) {
    super.update(changedProperties)
    if (changedProperties.has('doc')) {
      // destroy prosemirror view if it exists
      if (this.proseMirrorView) {
        this.proseMirrorView.destroy()
      }

      // Get the ProseMirror schema corresponding to the node type
      // of the document
      const tagName =
        this.renderRoot.querySelector('[root]')?.tagName.toLowerCase() ??
        'stencila-article'
      let schema: Schema
      let views: Record<string, NodeViewConstructor>
      if (tagName === 'stencila-article') {
        // eslint-disable-next-line no-extra-semi
        ;({ schema, views } = schemas.article)
      } else {
        throw new Error(`No schema for element '${tagName}'`)
      }

      // Parse the document's DOM into a ProseMirror document
      // and then remove it (because it will be redundant)
      const doc = DOMParser.fromSchema(schema).parse(this.renderRoot)
      this.renderRoot.firstElementChild.remove()

      this.proseMirrorClient = new ProseMirrorClient(
        this.doc,
        this.access,
        this.renderRoot as HTMLElement
      )

      this.proseMirrorView = new ProseMirrorView(this.renderRoot, {
        state: EditorState.create({
          doc,
        }),
        dispatchTransaction: this.proseMirrorClient.sendPatches(),
        nodeViews: views,
      })

      // Attach the `DomClient` to the ProseMirror element
      const proseMirrorElem = this.renderRoot.querySelector('.ProseMirror')
        .firstElementChild as HTMLElement
      this.domClient = new DomClient(this.doc, proseMirrorElem)
    }
  }

  override render() {
    return html`<stencila-article root></stencila-article>`
  }
}
