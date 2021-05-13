import { Component, h, Host, Prop, State } from '@stencil/core'
import { projects } from 'stencila'
import { CHANNEL } from '../../../../../preload/index'

type Project = projects.Project
type File = projects.File

@Component({
  tag: 'app-project-sidebar-files',
  styleUrl: 'app-project-sidebar-files.css',
  scoped: true,
})
export class AppProjectSidebarFiles {
  @Prop()
  projectDir: string

  @State()
  private project: Project | undefined

  private getFileList = (path: string) => {
    window.api.invoke(CHANNEL.GET_PROJECT_FILES, path).then((project) => {
      this.project = project as Project
    })
  }

  componentWillLoad() {
    return this.getFileList(`/${this.projectDir}`)
  }

  private formatPath = (file: File): string => {
    const relativeParent =
      (file.parent?.replace((this.project?.path ?? '') + '/', '') ?? '') + '/'

    return file.path.replace(relativeParent, '')
  }

  private pathToFileTree = (path: string) => {
    const file = this.project?.files[path]
    const isDir = file?.children !== undefined

    return (
      <li>
        <a
          href="#"
          class={{
            isDir,
            isFile: !isDir,
          }}
        >
          <stencila-icon icon={isDir ? 'folder' : 'file'}></stencila-icon>
          {file ? this.formatPath(file) : ''}
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
            {this.project?.files[this.project?.path]?.children?.map(
              this.pathToFileTree
            )}
          </ul>
        </div>
      </Host>
    )
  }
}
