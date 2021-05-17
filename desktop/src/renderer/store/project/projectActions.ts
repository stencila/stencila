import { CHANNEL } from '../../../preload'
import { Project } from '../../types'
import { store } from '../index'
import { projectSlice } from './projectStore'

export const getProjectDetails = (path: string) => {
  window.api.invoke(CHANNEL.GET_PROJECT_FILES, path).then((project) => {
    store.dispatch(projectSlice.actions.insert({ project: project as Project }))
  })
}
