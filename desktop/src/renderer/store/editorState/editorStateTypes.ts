import { EntityId } from '@reduxjs/toolkit'

export type EditorState = {
  id: EntityId
  state: Record<string, unknown>
}

export type EditorStateStore = Record<EntityId, EditorState | undefined>
