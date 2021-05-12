import { Component, h } from '@stencil/core'

@Component({
  tag: 'app-project-file-preview',
  styleUrl: 'app-project-file-preview.css',
  scoped: true,
})
export class AppProjectFilePreview {
  render() {
    return (
      <div class="app-project-file-preview">
        <pre>
          <code># File contents go here</code>
        </pre>
      </div>
    )
  }
}
