import {
  Action,
  combineReducers,
  configureStore,
  getDefaultMiddleware,
} from '@reduxjs/toolkit'
import { createStore } from '@stencil/store'
import { ThunkAction } from 'redux-thunk'
import { documentPaneSlice } from './documentPane/documentPaneStore'
import { projectSlice } from './project/projectStore'

// Placeholder for app config
const App = {}

const rootReducer = combineReducers({
  panes: documentPaneSlice.reducer,
  projects: projectSlice.reducer,
})

export type RootState = ReturnType<typeof rootReducer>

export const store = configureStore({
  reducer: rootReducer,
  middleware: getDefaultMiddleware({
    thunk: {
      extraArgument: App,
    },
  }),
})

export const { state, onChange } = createStore({
  panes: store.getState().panes,
  projects: store.getState().projects,
})

store.subscribe(() => {
  state.panes = store.getState().panes
  state.projects = store.getState().projects
})

export type AppState = ReturnType<typeof rootReducer>

export type AppThunk = ThunkAction<void, RootState, typeof App, Action<string>>
