import { createAsyncThunk } from '@reduxjs/toolkit'
import { normalize } from 'normalizr'
import { client } from '../../client'
import { CHANNEL } from '../../../preload/channels'
import { projectEntity } from './entities'
import { ProjectStoreEntities } from './ProjectStoreTypes'

const StoreKeys = {
  recentProjects: 'recentProjects',
}

export const fetchRecentProjects = (): string[] => {
  const paths = window.localStorage.getItem(StoreKeys.recentProjects) ?? '[]'
  try {
    const parsedPaths = JSON.parse(paths)
    if (Array.isArray(parsedPaths)) {
      return parsedPaths.slice(0, 9)
    } else {
      return []
    }
  } catch {
    return []
  }
}

const saveRecentProjects = (path: string) => {
  const existingPaths = fetchRecentProjects()
  const dedupedPaths = new Set([...existingPaths, path])
  window.localStorage.setItem(
    StoreKeys.recentProjects,
    JSON.stringify([...dedupedPaths])
  )
}

export const fetchProject = createAsyncThunk(
  'projects/fetchProject',
  async (path: string) => {
    const data = await client.projects.open(path)

    saveRecentProjects(path)

    const normalized = normalize<any, ProjectStoreEntities>(data, projectEntity)

    return normalized
  }
)
