import { Component, h, Host, Prop } from '@stencil/core'
import { state } from '../../../../store'
import { selectProjectRootFiles } from '../../../../store/project/projectSelectors'

@Component({
  tag: 'app-project-sidebar-files',
  styleUrl: 'app-project-sidebar-files.css',
  scoped: true,
})
export class AppProjectSidebarFiles {
  @Prop()
  projectDir: string

  render() {
    const files = selectProjectRootFiles(state)
    return (
      <Host class="customScrollbar">
        <div class="app-project-sidebar-files">
          {files && files.length > 0 ? (
            <ul>
              {files.map((filePath) => (
                <app-project-sidebar-file
                  filePath={filePath}
                ></app-project-sidebar-file>
              ))}
            </ul>
          ) : (
            <app-sidebar-empty>
              <stencila-icon icon="seedling"></stencila-icon>
              <h2>This project doesn't contain any files yet…</h2>
            </app-sidebar-empty>
          )}
        </div>
      </Host>
    )
  }
}
