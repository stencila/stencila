import { apply as AP, option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { FileEvent } from 'stencila'
import { CHANNEL } from '../../../../preload'
import { state, store } from '../../../store'
import { closeDocument } from '../../../store/documentPane/documentPaneActions'
import {
  selectActiveView,
  selectPaneId
} from '../../../store/documentPane/documentPaneSelectors'
import { projectActions } from '../../../store/project/projectStore'

export const listenForFileEvents = (_projectId: string) => {
  window.api.receive(CHANNEL.GET_PROJECT_FILES, event => {
    const e = event as FileEvent
    store.dispatch(projectActions.updateProjectFiles(e.files))
  })

  window.api.receive(CHANNEL.CLOSE_ACTIVE_DOCUMENT, () => {
    pipe(
      AP.sequenceT(O.Apply)(
        O.fromNullable(selectPaneId(state)),
        selectActiveView(state)
      ),
      O.map(([paneId, viewId]) => {
        closeDocument(paneId, viewId)
      })
    )
  })
}

export const removeFileEventListener = () => {
  window.api.removeAll(CHANNEL.GET_PROJECT_FILES)
  window.api.removeAll(CHANNEL.CLOSE_ACTIVE_DOCUMENT)
}
