import { Component, h } from '@stencil/core'

@Component({
  tag: 'project-file-preview',
  styleUrl: 'filePreview.css',
  scoped: true,
})
export class ProjectFilePreview {
  render() {
    return (
      <div class="project-file-preview">
        <pre>
          <code># File contents go here</code>
        </pre>
      </div>
    )
  }
}
