import { DocumentView } from '../types'

export const VIEWS: Record<Exclude<DocumentView, 'directory'>, string> = {
  dynamic: 'Live updating and interactive view',
  live: 'Live updating view',
  source: 'Source code view',
  split: 'Two panel split view',
  static: 'Fixed, read-only view',
  visual: 'Visual editor',
  // TODO: We don't really want this in the menu but
  // TypeScript complains without it.
  directory: 'Tree view of the directory',
}
