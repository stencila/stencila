import { EntityId } from '@reduxjs/toolkit'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { state } from '..'
import { EditorState } from './editorStateTypes'

export const editorStateById = (id: EntityId): O.Option<EditorState> =>
  pipe(state.editors, (editorStates) => editorStates[id], O.fromNullable)
