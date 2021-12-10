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

const getAvailableLanguages = (): Promise<string[]> =>
  client.projects.kernels.languages().then(({ value: languages }) => {
    return languages
  })

export const getExecutableFormats =
  (): Promise<FileFormatUtils.FileFormatMap> =>
    getAvailableLanguages().then((languages) => {
      return languages.reduce(
        (formats: FileFormatUtils.FileFormatMap, language) => {
          const format = FileFormatUtils.lookupFormat(language)
          return {
            ...formats,
            [format.name]: format,
          }
        },
        {}
      )
    })
