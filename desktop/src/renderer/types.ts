import { File as FileType, Project as ProjectType } from 'stencila'

type WithPath<T> = T & {
  path: string
}

export type File = FileType
export type Project = WithPath<ProjectType>
