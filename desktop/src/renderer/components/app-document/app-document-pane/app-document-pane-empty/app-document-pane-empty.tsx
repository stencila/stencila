import { Component, h, Host } from '@stencil/core'
import { i18n } from '../../../../../i18n'

@Component({
  tag: 'app-document-pane-empty',
  styleUrl: 'app-document-pane-empty.css',
  scoped: true,
})
export class AppDocumentPaneEmptyState {
  render() {
    return (
      <Host>
        <div class="documentPaneEmpty">
          <stencila-icon icon="file-add"></stencila-icon>
          <h2>{i18n.t('document.pane.empty')}</h2>
        </div>
      </Host>
    )
  }
}
