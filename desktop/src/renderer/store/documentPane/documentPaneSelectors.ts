import { RootState } from '..'

export const selectPaneId = (state: RootState) => {
  return state.panes.ids[0]
}

export const selectPane = (state: RootState) => {
  const id = state.panes.ids[0]
  if (id) {
    return state.panes.entities[id]
  }
}
