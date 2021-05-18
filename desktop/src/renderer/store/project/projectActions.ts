import { createAsyncThunk } from '@reduxjs/toolkit'
import { normalize } from 'normalizr'
import { CHANNEL } from '../../../preload'
import { projectEntity } from './entities'
import { ProjectStoreEntities } from './projectStore'

export const fetchProject = createAsyncThunk(
  'projects/fetchProject',
  async (path: string) => {
    const data = await window.api.invoke(CHANNEL.GET_PROJECT_FILES, path)

    const normalized = normalize<any, ProjectStoreEntities>(data, projectEntity)

    return normalized
  }
)
