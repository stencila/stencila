import { createContext } from '@lit/context'

import { DocumentView } from '../types'

export type SidebarContext = {
  filesOpen?: boolean
  view?: DocumentView
}

export const sidebarContext = createContext<SidebarContext>('sidebar')
