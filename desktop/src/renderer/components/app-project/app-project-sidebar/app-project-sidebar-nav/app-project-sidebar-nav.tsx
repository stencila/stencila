import { Component, h, Host } from '@stencil/core'
import { option as O, taskEither as TE } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import {
  createNewDocument,
  openDocumentInActivePane,
} from '../../../../store/documentPane/documentPaneActions'
import { sessionStore } from '../../../../store/sessionStore'
import { userOSPathSeparator } from '../../../../utils/env'
import { errorToast } from '../../../../utils/errors'
import { ProjectRouter } from '../../projectRouter'

/**
 * Attempt to open the Project specific settings file, or create it if one doesn't exist
 */
const openOrInitProjectSettings = (path: string) => {
  const configPath = [path, 'project.json'].join(userOSPathSeparator)

  pipe(
    configPath,
    openDocumentInActivePane,
    TE.mapLeft((err) => {
      if (typeof err === 'string' && err.includes('No such file')) {
        createNewDocument(configPath, 'json')
      }
    }),
    TE.mapLeft(errorToast)
  )()
}

@Component({
  tag: 'app-project-sidebar-nav',
  styleUrl: 'app-project-sidebar-nav.css',
  scoped: true,
})
export class AppProjectSidebarNav {
  render() {
    return (
      <Host>
        <div>
          <stencila-button
            icon="file"
            minimal={true}
            color="stock"
            iconOnly={true}
            tooltip="Project Files"
            onClick={(e) => {
              e.preventDefault()
              ProjectRouter.push('/project/')
            }}
          ></stencila-button>

          <stencila-button
            icon="stackshare"
            minimal={true}
            color="stock"
            iconOnly={true}
            tooltip="Project Graph"
            onClick={(e) => {
              e.preventDefault()
              ProjectRouter.push('/project/graph')
            }}
          ></stencila-button>
        </div>

        <div>
          <stencila-button
            icon="settings-3"
            minimal={true}
            color="stock"
            iconOnly={true}
            tooltip="Project Settings"
            onClick={(e) => {
              e.preventDefault()
              ProjectRouter.push('/project/').then(() => {
                pipe(
                  sessionStore.get('PROJECT_PATH'),
                  O.map((projectPath) => {
                    openOrInitProjectSettings(projectPath)
                  })
                )
              })
            }}
          ></stencila-button>
        </div>
      </Host>
    )
  }
}
