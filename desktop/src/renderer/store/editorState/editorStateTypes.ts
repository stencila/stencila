import { EditorState as CodeMirrorState } from '@codemirror/state'
import { EntityId } from '@reduxjs/toolkit'

export type EditorState = {
  id: EntityId
  state: CodeMirrorState
}

export type EditorStateStore = Record<EntityId, EditorState | undefined>
