import { EntityId } from '@reduxjs/toolkit'
import { File } from '../../types'
import { NormalizedProject } from './entities'

export type ProjectStoreEntities = {
  projects: Record<EntityId, NormalizedProject | undefined>
  files: Record<EntityId, File | undefined>
}

export type NormalizedProjectStore = {
  entities: ProjectStoreEntities
  ids: string[]
}
