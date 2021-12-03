import { createAsyncThunk } from '@reduxjs/toolkit'
import { FileFormatUtils } from '@stencila/components'
import { normalize } from 'normalizr'
import { client } from '../../client'
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
      return parsedPaths.slice(0, 6)
    } else {
      return []
    }
  } catch {
    return []
  }
}

const saveRecentProjects = (path: string) => {
  const existingPaths = fetchRecentProjects()
  const dedupedPaths = new Set([path, ...existingPaths])
  window.localStorage.setItem(
    StoreKeys.recentProjects,
    JSON.stringify([...dedupedPaths])
  )
}

export const fetchProject = createAsyncThunk(
  'projects/fetchProject',
  async (path: string) => {
    saveRecentProjects(path)

    const { value: project } = await client.projects.contents(path)

    const normalized = normalize<any, ProjectStoreEntities>(
      project,
      projectEntity
    )

    return normalized
  }
)

const getAvailableKernels = (): Promise<string[]> =>
  client.projects.kernels.available().then(({ value: kernels }) => {
    return kernels
  })

export const getExecutableFormats =
  (): Promise<FileFormatUtils.FileFormatMap> =>
    getAvailableKernels().then((kernels) => {
      return kernels.reduce(
        (formats: FileFormatUtils.FileFormatMap, kernel) => {
          const format = FileFormatUtils.lookupFormat(kernel)
          return {
            ...formats,
            [format.name]: format,
          }
        },
        {}
      )
    })
