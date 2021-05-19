import { File } from 'stencila'
import { CHANNEL } from '../../../../preload'
import { store } from '../../../store'
import { projectActions } from '../../../store/project/projectStore'

interface ProjectEvent {
  file: File | null
  files: Record<string, File>
  // TODO: See if `kind` can be made into an enum
  kind: string
  path: string
  project: string
}

export const listenForProjectEvents = () => {
  window.api.receive(CHANNEL.GET_PROJECT_FILES, (event) => {
    const e = event as ProjectEvent
    store.dispatch(projectActions.updateProjectFiles(e.files))
  })
}
