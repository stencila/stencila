import { EntityId } from '@reduxjs/toolkit'
import { state } from '..'
import { EditorState } from './editorStateTypes'

export const saveEditorState =
  (id: EntityId) =>
  (editorState: EditorState): EditorState => {
    state.editors[id] = editorState
    return editorState
  }

export const clearEditorState = (id: EntityId): void => {
  delete state.editors[id]
}
