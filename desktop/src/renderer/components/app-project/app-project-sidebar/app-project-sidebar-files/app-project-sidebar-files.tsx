import { Component, h, Host, Prop } from '@stencil/core'
import { state } from '../../../../store'
import { setActiveDocument } from '../../../../store/documentPane/documentPaneActions'
import { selectPaneId } from '../../../../store/documentPane/documentPaneSelectors'
import {
  selectProjectFile,
  selectProjectRootFiles,
} from '../../../../store/project/projectSelectors'
import { getFileIcon } from './iconMap'

@Component({
  tag: 'app-project-sidebar-files',
  styleUrl: 'app-project-sidebar-files.css',
  scoped: true,
})
export class AppProjectSidebarFiles {
  @Prop()
  projectDir: string

  setActiveFile = (path: string) => {
    const paneId = selectPaneId(state)
    if (paneId) {
      setActiveDocument(paneId.toString(), path)
    }
  }

  private pathToFileTree = (path: string) => {
    const file = selectProjectFile(state)(path)

    if (!file) return

    const isDir = file?.children !== undefined

    return (
      <li>
        <a
          href="#"
          class={{
            isDir,
            isFile: !isDir,
          }}
          onClick={(e: MouseEvent) => {
            e.preventDefault()
            if (!isDir) {
              this.setActiveFile(path)
            }
          }}
        >
          <stencila-icon icon={getFileIcon(file)}></stencila-icon>
          {file?.name}
        </a>
        {file?.children && <ul>{file.children.map(this.pathToFileTree)}</ul>}
      </li>
    )
  }

  render() {
    const files = selectProjectRootFiles(state)
    return (
      <Host class="customScrollbar">
        <div class="app-project-sidebar-files">
          {files && files.length > 0 ? (
            <ul>{files.map(this.pathToFileTree)}</ul>
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
