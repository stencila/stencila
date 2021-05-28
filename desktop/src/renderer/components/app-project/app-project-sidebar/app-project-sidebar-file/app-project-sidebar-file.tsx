import { Component, h, Host, Prop, State } from '@stencil/core'
import { File } from 'stencila'
import { state } from '../../../../store'
import { setActiveDocument } from '../../../../store/documentPane/documentPaneActions'
import { selectPaneId } from '../../../../store/documentPane/documentPaneSelectors'
import { selectProjectFile } from '../../../../store/project/projectSelectors'
import { getFileIcon } from './iconMap'

@Component({
  tag: 'app-project-sidebar-file',
  styleUrl: 'app-project-sidebar-file.css',
  scoped: true,
})
export class AppProjectSidebarFile {
  @Prop()
  filePath: string

  @State()
  isCollapsed = true

  private file: File | undefined

  setActiveFile = (path: string) => {
    const paneId = selectPaneId(state)
    if (paneId) {
      setActiveDocument(paneId.toString(), path)
    }
  }

  private clickHandler = (e: MouseEvent) => {
    e.preventDefault()

    if (this.file?.children) {
      this.isCollapsed = !this.isCollapsed
    } else {
      this.setActiveFile(this.filePath)
    }
  }

  componentWillLoad() {
    this.file = selectProjectFile(state)(this.filePath)
  }

  render() {
    const file = selectProjectFile(state)(this.filePath)

    if (!file) return

    const isDir = file?.children !== undefined

    return (
      <Host>
        <li>
          <a
            href="#"
            class={{
              isDir,
              isFile: !isDir,
            }}
            onClick={this.clickHandler}
          >
            <stencila-icon
              icon={getFileIcon(file, this.isCollapsed)}
            ></stencila-icon>
            {file?.name}
          </a>
          {!this.isCollapsed && file?.children && (
            <ul>
              {file.children.map((filePath) => (
                <app-project-sidebar-file
                  filePath={filePath}
                ></app-project-sidebar-file>
              ))}
            </ul>
          )}
        </li>
      </Host>
    )
  }
}
