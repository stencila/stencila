import { Component, h, Host } from '@stencil/core'

@Component({
  tag: 'app-sidebar-empty',
  styleUrl: 'app-sidebar-empty.css',
  scoped: true,
})
export class AppSidebarEmptyState {
  render() {
    return (
      <Host>
        <div class="app-sidebar-empty">
          <slot />
        </div>
      </Host>
    )
  }
}
