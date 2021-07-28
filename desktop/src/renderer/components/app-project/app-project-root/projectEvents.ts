import { apply as AP, option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { FileEvent } from 'stencila'
import { CHANNEL } from '../../../../preload/channels'
import { state, store } from '../../../store'
import {
  closeDocument,
  createNewDocument,
} from '../../../store/documentPane/documentPaneActions'
import {
  selectActiveView,
  selectPaneId,
} from '../../../store/documentPane/documentPaneSelectors'
import { projectActions } from '../../../store/project/projectStore'

export const listenForFileEvents = (_projectId: string) => {
  window.api.receive(CHANNEL.PROJECTS_OPEN, (event) => {
    const e = event as FileEvent
    store.dispatch(projectActions.updateProjectFiles(e.files))
  })

  window.api.receive(CHANNEL.DOCUMENTS_CREATE, createNewDocument)

  window.api.receive(CHANNEL.DOCUMENTS_CLOSE_ACTIVE, () => {
    pipe(
      AP.sequenceT(O.Apply)(selectPaneId(state), selectActiveView(state)),
      O.map(([paneId, viewId]) => {
        closeDocument(paneId, viewId)
      })
    )
  })
}

export const removeFileEventListener = () => {
  window.api.removeAll(CHANNEL.PROJECTS_OPEN)
  window.api.removeAll(CHANNEL.DOCUMENTS_CLOSE_ACTIVE)
}
