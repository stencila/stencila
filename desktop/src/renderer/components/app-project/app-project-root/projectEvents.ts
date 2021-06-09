import { FileEvent } from 'stencila'
import { CHANNEL } from '../../../../preload'
import { store } from '../../../store'
import { projectActions } from '../../../store/project/projectStore'

export const listenForFileEvents = () => {
  window.api.receive(CHANNEL.GET_PROJECT_FILES, (event) => {
    const e = event as FileEvent
    store.dispatch(projectActions.updateProjectFiles(e.files))
  })
}

export const removeFileEventListener = () => {
  window.api.removeAll(CHANNEL.GET_PROJECT_FILES)
}
