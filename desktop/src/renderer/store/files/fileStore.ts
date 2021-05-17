import { createEntityAdapter, createSlice } from '@reduxjs/toolkit'
import { File } from '../../types'

const fileAdapter = createEntityAdapter<File>()

const initialState = fileAdapter.getInitialState()

export const fileSlice = createSlice({
  name: 'file',
  initialState,
  reducers: {},
})

export const fileActions = fileSlice.actions
export type FileActions = typeof fileActions
