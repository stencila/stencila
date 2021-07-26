import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop } from '@stencil/core'
import { TogglePaneLayoutButton } from '../actions/toggleLayout'

@Component({
  tag: 'app-document-pane-action-bar',
  styleUrl: 'app-document-pane-action-bar.css',
  scoped: true,
})
export class AppDocumentPaneActionBar {
  @Prop() paneId!: EntityId

  @Prop() docId!: EntityId

  render() {
    return (
      <Host>
        <div class="leadingActions"></div>

        <div class="trailingActions">
          <TogglePaneLayoutButton
            paneId={this.paneId}
            viewId={this.docId}
          ></TogglePaneLayoutButton>
        </div>
      </Host>
    )
  }
}
