import { Component, h, Host, Prop } from '@stencil/core'
import { state } from '../../../../store'
import { setActiveDocument } from '../../../../store/documentPane/documentPaneActions'
import { selectPaneId } from '../../../../store/documentPane/documentPaneSelectors'
import {
  selectProject,
  selectProjectFiles,
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
    const files = selectProject(state)?.files

    if (!files) return

    const file = files[path]
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
    return (
      <Host>
        <div class="app-project-sidebar-files">
          <ul>
            {selectProjectFiles(state)?.children?.map(this.pathToFileTree)}
          </ul>
        </div>
      </Host>
    )
  }
}
