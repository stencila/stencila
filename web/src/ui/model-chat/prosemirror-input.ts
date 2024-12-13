import { LitElement, html, PropertyValues } from 'lit'
import { customElement } from 'lit/decorators.js'
import { schema } from 'prosemirror-schema-basic'
import { EditorState } from 'prosemirror-state'
import { EditorView } from 'prosemirror-view'

// Import the required CSS
import 'prosemirror-view/style/prosemirror.css'
import 'prosemirror-menu/style/menu.css'

// TODO: get styles loading in
@customElement('stencila-ui-model-chat-text-input')
export class UiModelChatUserInput extends LitElement {
  private view?: EditorView

  protected override update(changedProperties: PropertyValues): void {
    super.update(changedProperties)

    if (this.view) {
      this.view.destroy()
    }

    const state = EditorState.create({
      schema: schema,
    })

    this.view = new EditorView(this.renderRoot.querySelector('#editor'), {
      state,
    })
  }

  public getContent() {
    this.view.state.doc.toString()
  }

  override render() {
    return html`<div id="editor"></div>`
  }

  override disconnectedCallback() {
    this.view?.destroy()
    super.disconnectedCallback()
  }
}
