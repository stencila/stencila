import { EntityId } from '@reduxjs/toolkit'
import { option as O } from 'fp-ts'
import { Document } from 'stencila'

export type PaneModule = 'editor' | 'preview'

export type PaneLayout = {
  id: EntityId
  modules: PaneModule[]
  orientation: 'horizontal' | 'vertical'
  sizes: number[]
}

export type DocumentPane = {
  id: EntityId
  activeView: O.Option<EntityId>
  views: EntityId[]
}

export type NormalizedDocumentPaneStore = {
  activePane: O.Option<EntityId>
  entities: {
    panes: Record<EntityId, DocumentPane>
    layouts: Record<EntityId, PaneLayout>
    views: Record<EntityId, Document>
  }
  ids: EntityId[]
}
